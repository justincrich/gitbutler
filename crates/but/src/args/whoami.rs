/// Arguments for the `but whoami` self-scoped discovery command.
#[derive(Debug, clap::Parser)]
pub struct Platform {
    /// Principal to inspect. Defaults to the caller resolved from BUT_AGENT_HANDLE.
    /// Targeting another principal requires administration:read.
    #[clap(long)]
    pub principal: Option<String>,
}
