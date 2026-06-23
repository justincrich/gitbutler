/// Arguments for the `but can-i` authority-hold check.
#[derive(Debug, clap::Parser)]
pub struct Platform {
    /// Functional authority token to check (e.g. "merge", "reviews:write").
    pub authority: String,
    /// Principal to check against. Defaults to the caller from BUT_AGENT_HANDLE.
    /// Targeting another principal requires administration:read.
    #[clap(long)]
    pub principal: Option<String>,
}
