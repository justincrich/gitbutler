//! Process identity helpers for binding authorization state to an OS process.
//!
//! # Source
//!
//! ## Linux
//!
//! Process start time comes from `/proc/[pid]/stat` field 22, `starttime`,
//! which is measured in clock ticks after system boot. Seconds since epoch are
//! `btime + (starttime / sysconf(_SC_CLK_TCK))`, where `btime` comes from
//! `/proc/stat`.
//!
//! ## macOS
//!
//! Process start time comes from `proc_bsdinfo.pbs_start` returned by
//! `proc_pidinfo(pid, PROC_PIDTBSDINFO, ...)`. In `libc 0.2.186`, the Darwin
//! binding exposes that value as `proc_bsdinfo::pbi_start_tvsec` and
//! `proc_bsdinfo::pbi_start_tvusec`; `process_start_time` returns the seconds
//! field because this crate's canonical form is Unix seconds.

/// Return the current operating-system process id.
pub fn current_pid() -> u32 {
    std::process::id()
}

/// Return the process start time as Unix seconds.
pub fn process_start_time(pid: u32) -> anyhow::Result<u64> {
    platform::process_start_time(pid)
}

#[cfg(target_os = "linux")]
mod platform {
    use std::fs;

    use anyhow::{Context, bail, ensure};

    const STARTTIME_INDEX_AFTER_COMM: usize = 19;

    pub(super) fn process_start_time(pid: u32) -> anyhow::Result<u64> {
        let stat_path = format!("/proc/{pid}/stat");
        let stat = fs::read_to_string(&stat_path)
            .with_context(|| format!("failed to read {stat_path} for pid {pid}"))?;
        let start_ticks = parse_start_ticks(&stat, pid)?;
        let ticks_per_second = clock_ticks_per_second()?;
        let boot_time = boot_time_seconds()?;
        let seconds_since_boot = start_ticks / ticks_per_second;

        boot_time
            .checked_add(seconds_since_boot)
            .with_context(|| format!("process start time for pid {pid} overflowed u64 seconds"))
    }

    fn parse_start_ticks(stat: &str, pid: u32) -> anyhow::Result<u64> {
        let comm_start = stat
            .find('(')
            .with_context(|| format!("malformed /proc stat for pid {pid}: missing comm start"))?;
        let comm_end = stat
            .rfind(')')
            .with_context(|| format!("malformed /proc stat for pid {pid}: missing comm end"))?;
        ensure!(
            comm_end > comm_start,
            "malformed /proc stat for pid {pid}: comm field is invalid"
        );

        let stat_pid = stat[..comm_start]
            .trim()
            .parse::<u32>()
            .with_context(|| format!("malformed /proc stat for pid {pid}: invalid pid field"))?;
        ensure!(
            stat_pid == pid,
            "read /proc stat for pid {pid}, but stat payload reported pid {stat_pid}"
        );

        stat[comm_end + 1..]
            .split_whitespace()
            .nth(STARTTIME_INDEX_AFTER_COMM)
            .with_context(|| {
                format!("malformed /proc stat for pid {pid}: missing field 22 starttime")
            })?
            .parse::<u64>()
            .with_context(|| {
                format!("malformed /proc stat for pid {pid}: field 22 starttime is not a number")
            })
    }

    fn clock_ticks_per_second() -> anyhow::Result<u64> {
        // SAFETY: sysconf is thread-safe for _SC_CLK_TCK and does not require
        // pointers or ownership transfer.
        let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        if ticks <= 0 {
            bail!(
                "failed to read clock ticks per second with sysconf(_SC_CLK_TCK): {}",
                std::io::Error::last_os_error()
            );
        }

        Ok(ticks as u64)
    }

    fn boot_time_seconds() -> anyhow::Result<u64> {
        let stat = fs::read_to_string("/proc/stat")
            .context("failed to read /proc/stat for system boot time")?;

        for line in stat.lines() {
            let mut fields = line.split_whitespace();
            if fields.next() == Some("btime") {
                return fields
                    .next()
                    .context("malformed /proc/stat: btime is missing its value")?
                    .parse::<u64>()
                    .context("malformed /proc/stat: btime is not a number");
            }
        }

        bail!("failed to find btime in /proc/stat")
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::mem::MaybeUninit;

    use anyhow::{Context, bail, ensure};

    pub(super) fn process_start_time(pid: u32) -> anyhow::Result<u64> {
        let pid_t: libc::c_int = pid
            .try_into()
            .with_context(|| format!("pid {pid} cannot be represented as macOS pid_t"))?;
        let mut info = MaybeUninit::<libc::proc_bsdinfo>::zeroed();
        let expected_size = libc::c_int::try_from(std::mem::size_of::<libc::proc_bsdinfo>())
            .context("proc_bsdinfo size does not fit in libc::c_int")?;

        // SAFETY: `info` points to writable storage large enough for
        // proc_bsdinfo, and proc_pidinfo initializes it when it returns the
        // expected byte count.
        let result = unsafe {
            libc::proc_pidinfo(
                pid_t,
                libc::PROC_PIDTBSDINFO,
                0,
                info.as_mut_ptr().cast(),
                expected_size,
            )
        };
        if result != expected_size {
            bail!(
                "failed to inspect process start time for pid {pid} with proc_pidinfo(PROC_PIDTBSDINFO): returned {result} bytes, expected {expected_size}: {}",
                std::io::Error::last_os_error()
            );
        }

        // SAFETY: proc_pidinfo returned the full proc_bsdinfo byte count above.
        let info = unsafe { info.assume_init() };
        ensure!(
            info.pbi_pid == pid,
            "proc_pidinfo(PROC_PIDTBSDINFO) returned pid {} while inspecting pid {pid}",
            info.pbi_pid
        );
        ensure!(
            info.pbi_start_tvsec != 0,
            "proc_pidinfo(PROC_PIDTBSDINFO) returned zero start time for pid {pid}"
        );

        Ok(info.pbi_start_tvsec)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod platform {
    use anyhow::bail;

    pub(super) fn process_start_time(_pid: u32) -> anyhow::Result<u64> {
        bail!(
            "process_start_time: unsupported platform: {}",
            std::env::consts::OS
        )
    }
}
