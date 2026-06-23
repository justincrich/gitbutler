/// Arguments for the `but perm` command and subcommands.

#[derive(Debug, clap::Parser)]
pub struct Platform {
    #[clap(subcommand)]
    pub cmd: Option<Subcommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// List effective permissions for a principal.
    List {
        /// Principal to inspect. Defaults to the caller resolved from BUT_AGENT_HANDLE.
        #[clap(long)]
        principal: Option<String>,
    },
    /// Grant direct permissions to a principal in the working-tree governance config.
    Grant {
        /// Principal to grant.
        #[clap(long, required = true)]
        principal: String,
        /// Functional permission tokens to grant.
        #[clap(required = true)]
        authorities: Vec<String>,
    },
    /// Revoke direct permissions from a principal in the working-tree governance config.
    Revoke {
        /// Principal to revoke from.
        #[clap(long, required = true)]
        principal: String,
        /// Functional permission tokens to revoke.
        #[clap(required = true)]
        authorities: Vec<String>,
    },
}
