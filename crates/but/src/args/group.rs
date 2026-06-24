/// Arguments for the `but group` command and subcommands.

#[derive(Debug, clap::Parser)]
pub struct Platform {
    #[clap(subcommand)]
    pub cmd: Option<Subcommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// List governed groups, grants, and members.
    List,
    /// Create a governed group in the working-tree governance config.
    Create {
        /// Group name to create.
        name: String,
        /// Initial functional permission tokens to grant to the group.
        #[clap(long = "permissions", required = true, num_args = 1..)]
        authorities: Vec<String>,
    },
    /// Grant functional permissions to a group in the working-tree governance config.
    Grant {
        /// Group name to grant.
        name: String,
        /// Functional permission tokens to grant.
        #[clap(required = true)]
        authorities: Vec<String>,
    },
    /// Add a principal to a group in the working-tree governance config.
    AddMember {
        /// Group name to update.
        name: String,
        /// Principal to add as a member.
        member: String,
    },
    /// Remove a principal from a group in the working-tree governance config.
    RemoveMember {
        /// Group name to update.
        name: String,
        /// Principal to remove from the members list.
        member: String,
    },
    /// Delete a governed group from the working-tree governance config.
    Delete {
        /// Group name to delete.
        name: String,
    },
}
