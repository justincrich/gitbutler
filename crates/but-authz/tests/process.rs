use std::time::{SystemTime, UNIX_EPOCH};

use but_authz::{current_pid, process_start_time};

#[allow(non_snake_case)]
#[test]
fn IDENT_002_current_pid_matches_std_process_id() {
    assert_eq!(
        current_pid(),
        std::process::id(),
        "current_pid must report the operating-system pid of this test process"
    );
}

#[allow(non_snake_case)]
#[test]
fn IDENT_002_current_process_start_time_is_sane_unix_timestamp() -> anyhow::Result<()> {
    let pid = current_pid();
    let started_at = process_start_time(pid)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the unix epoch")
        .as_secs();

    assert!(
        started_at > 1_000_000_000,
        "process_start_time({pid}) must return unix seconds, not zero, jiffies, or ticks"
    );
    assert!(
        started_at <= now,
        "process_start_time({pid}) must not be in the future"
    );

    Ok(())
}

#[allow(non_snake_case)]
#[test]
fn IDENT_002_current_process_start_time_is_stable_across_reads() -> anyhow::Result<()> {
    let pid = current_pid();
    let first = process_start_time(pid)?;
    let second = process_start_time(pid)?;

    assert_eq!(
        first, second,
        "process_start_time({pid}) must be stable for the lifetime of a process"
    );

    Ok(())
}

#[allow(non_snake_case)]
#[test]
fn IDENT_002_nonexistent_pid_returns_error_that_names_pid() {
    let pid = u32::MAX;
    let error = process_start_time(pid).expect_err("nonexistent pid must return Err");
    let message = error.to_string();

    assert!(
        message.contains(&pid.to_string()),
        "process_start_time({pid}) error must name the pid; got {message:?}"
    );
}

#[allow(non_snake_case)]
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn IDENT_002_unsupported_platform_returns_error_naming_platform() {
    let error = process_start_time(1234).expect_err("unsupported platform must return Err");
    let message = error.to_string();

    assert!(
        message.contains("unsupported platform"),
        "unsupported-platform error must explain the platform is unsupported; got {message:?}"
    );
    assert!(
        message.contains(std::env::consts::OS),
        "unsupported-platform error must name the target OS {}; got {message:?}",
        std::env::consts::OS
    );
}
