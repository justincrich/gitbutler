/// Arguments for the `but agent` command and subcommands.

#[derive(Debug, clap::Parser)]
pub struct Platform {
    #[clap(subcommand)]
    pub cmd: Option<Subcommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Register a process as an agent.
    Register {
        /// Process id to register. Defaults to this `but` process.
        #[clap(long)]
        pid: Option<u32>,
        /// Process start time as Unix seconds.
        #[clap(long)]
        start_time: Option<u64>,
        /// Committed agent id to register as.
        #[clap(long = "as")]
        r#as: String,
        /// Registration TTL: seconds, Ns, Nm, or Nh.
        #[clap(long, default_value = "4h", value_parser = parse_ttl_seconds)]
        ttl: u64,
        /// Agent id that performed the registration.
        #[clap(long, default_value = "operator")]
        by: String,
    },
    /// Unregister a process. Missing registrations are not an error.
    Unregister {
        /// Process id to unregister.
        #[clap(long)]
        pid: u32,
        /// Process start time as Unix seconds. If omitted, all entries for the pid are removed.
        #[clap(long)]
        start_time: Option<u64>,
    },
    /// List live runtime registrations, or the committed roster with --committed.
    List {
        /// Read committed `.gitbutler/agents.toml` instead of the runtime registry.
        #[clap(long)]
        committed: bool,
    },
    /// Print the committed agent id for this process.
    Whoami,
}

fn parse_ttl_seconds(value: &str) -> Result<u64, String> {
    let (digits, multiplier) = if let Some(seconds) = value.strip_suffix('s') {
        (seconds, 1_u64)
    } else if let Some(minutes) = value.strip_suffix('m') {
        (minutes, 60_u64)
    } else if let Some(hours) = value.strip_suffix('h') {
        (hours, 60_u64 * 60)
    } else if value.chars().all(|c| c.is_ascii_digit()) {
        (value, 1_u64)
    } else {
        return Err("TTL must be seconds, Ns, Nm, or Nh".to_owned());
    };

    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err("TTL duration must start with a positive integer".to_owned());
    }

    let amount = digits
        .parse::<u64>()
        .map_err(|_| "TTL duration is too large".to_owned())?;
    if amount == 0 {
        return Err("TTL duration must be greater than zero".to_owned());
    }

    amount
        .checked_mul(multiplier)
        .ok_or_else(|| "TTL duration is too large".to_owned())
}
