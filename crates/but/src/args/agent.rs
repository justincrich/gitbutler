/// Arguments for the `but agent` command and subcommands.

#[derive(Debug, clap::Parser)]
pub struct Platform {
    #[clap(subcommand)]
    pub cmd: Option<Subcommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// List the committed agent roster from `.gitbutler/agents.toml`.
    List {
        /// Read committed `.gitbutler/agents.toml` (the only supported source).
        #[clap(long)]
        committed: bool,
    },
    /// Rewrite working-tree `.gitbutler/permissions.toml` to `.gitbutler/agents.toml`.
    Migrate,
}
