//! # Feature Flags
#![cfg_attr(
    not(feature = "document-features"),
    doc = "Activate the `document-features` cargo feature to see feature docs here, i.e. `cargo doc -p gitbutler-tauri --features document-features`"
)]
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
#![cfg_attr(
    all(windows, not(test), not(debug_assertions)),
    windows_subsystem = "windows"
)]
// FIXME(qix-): Stuff we want to fix but don't have a lot of time for.
// FIXME(qix-): PRs welcome!
#![allow(
    clippy::used_underscore_binding,
    clippy::module_name_repetitions,
    clippy::struct_field_names,
    clippy::too_many_lines
)]

pub mod broadcaster;
#[cfg(feature = "irc")]
pub mod irc;
#[cfg(feature = "irc")]
pub mod irc_lifecycle;

pub mod logs;
pub mod menu;
pub mod window;
pub use window::state::{WindowState, event::ChangeForFrontend};

pub mod action;
pub mod askpass;
pub mod debug;
pub mod projects;

pub mod settings;
pub mod zip;

pub mod env;
pub mod governance;

pub mod csp;

use but_api::{branch, commit, diff, github, gitlab, legacy, open, platform, workspace};

/// Governance commands registered through the but-api boundary.
pub const GOVERNANCE_COMMANDS: &[&str] = &[
    "perm_list",
    "perm_grant",
    "perm_revoke",
    "group_create",
    "group_grant",
    "group_revoke",
    "group_add_member",
    "group_remove_member",
    "group_delete",
    "group_list",
    "branch_gates_read",
    "branch_gates_update",
    "governance_status_read",
    "governance_principals_list",
    "governance_pending",
    "governance_commit",
];

/// Expands the production governance/config-management command rows into a caller-supplied macro.
///
/// Production `invoke_handler()` and IPC registration tests both consume this
/// macro, which keeps the registration paths in one place while allowing tests
/// to run with Tauri's mock runtime.
#[macro_export]
macro_rules! gitbutler_governance_command_rows {
    ($handler:ident) => {
        $handler![
            but_api::legacy::governance::tauri_group_list::group_list,
            $crate::governance::tauri_group_create::group_create,
            $crate::governance::tauri_group_grant::group_grant,
            $crate::governance::tauri_group_revoke::group_revoke,
            $crate::governance::tauri_group_add_member::group_add_member,
            $crate::governance::tauri_group_remove_member::group_remove_member,
            $crate::governance::tauri_group_delete::group_delete,
            but_api::legacy::governance::tauri_perm_list::perm_list,
            $crate::governance::tauri_perm_grant::perm_grant,
            $crate::governance::tauri_perm_revoke::perm_revoke,
            but_api::legacy::governance::tauri_branch_gates_read::branch_gates_read,
            $crate::governance::tauri_branch_gates_update::branch_gates_update,
            but_api::legacy::governance::tauri_governance_status_read::governance_status_read,
            $crate::governance::tauri_governance_principals_list::governance_principals_list,
            $crate::governance::tauri_governance_pending::governance_pending,
            $crate::governance::tauri_governance_commit::governance_commit,
            $crate::governance::tauri_agent_perm_grant::agent_perm_grant,
        ]
    };
}

pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    macro_rules! full_invoke_handler {
        ($($governance_command:path),* $(,)?) => {
            tauri::generate_handler![
        github::tauri_init_github_device_oauth::init_github_device_oauth,
        github::tauri_check_github_auth_status::check_github_auth_status,
        github::tauri_store_github_pat::store_github_pat,
        github::tauri_store_github_enterprise_pat::store_github_enterprise_pat,
        github::tauri_get_gh_user::get_gh_user,
        github::tauri_forget_github_account::forget_github_account,
        github::tauri_list_known_github_accounts::list_known_github_accounts,
        github::tauri_clear_all_github_tokens::clear_all_github_tokens,
        gitlab::tauri_store_gitlab_pat::store_gitlab_pat,
        gitlab::tauri_store_gitlab_selfhosted_pat::store_gitlab_selfhosted_pat,
        gitlab::tauri_get_gl_user::get_gl_user,
        gitlab::tauri_forget_gitlab_account::forget_gitlab_account,
        gitlab::tauri_list_known_gitlab_accounts::list_known_gitlab_accounts,
        gitlab::tauri_clear_all_gitlab_tokens::clear_all_gitlab_tokens,
        diff::tauri_commit_details::commit_details,
        diff::tauri_commit_details_with_line_stats::commit_details_with_line_stats,
        but_api::branch::tauri_branch_diff::branch_diff,
        but_api::branch::tauri_move_branch::move_branch,
        but_api::branch::tauri_tear_off_branch::tear_off_branch,
        legacy::git::tauri_git_remote_branches::git_remote_branches,
        legacy::git::tauri_delete_all_data::delete_all_data,
        legacy::git::tauri_git_set_global_config::git_set_global_config,
        legacy::git::tauri_git_remove_global_config::git_remove_global_config,
        legacy::git::tauri_git_get_global_config::git_get_global_config,
        legacy::git::tauri_git_test_push::git_test_push,
        legacy::git::tauri_git_test_fetch::git_test_fetch,
        legacy::git::tauri_git_index_size::git_index_size,
        legacy::users::tauri_set_user::set_user,
        legacy::users::tauri_delete_user::delete_user,
        legacy::users::tauri_get_user::get_user,
        legacy::users::tauri_get_login_token::get_login_token,
        legacy::users::tauri_login_with_token::login_with_token,
        legacy::users::tauri_get_user_profile::get_user_profile,
        legacy::users::tauri_update_user_profile::update_user_profile,
        legacy::projects::tauri_add_project::add_project,
        legacy::projects::tauri_add_project_best_effort::add_project_best_effort,
        legacy::projects::tauri_get_project::get_project,
        legacy::projects::tauri_update_project::update_project,
        legacy::projects::tauri_delete_project::delete_project,
        legacy::projects::tauri_is_gerrit::is_gerrit,
        legacy::repo::tauri_check_signing_settings::check_signing_settings,
        legacy::repo::tauri_git_clone_repository::git_clone_repository,
        legacy::repo::tauri_get_commit_file::get_commit_file,
        legacy::repo::tauri_get_workspace_file::get_workspace_file,
        legacy::repo::tauri_get_blob_file::get_blob_file,
        legacy::repo::tauri_find_files::find_files,
        legacy::repo::tauri_pre_commit_hook_diffspecs::pre_commit_hook_diffspecs,
        legacy::repo::tauri_post_commit_hook::post_commit_hook,
        legacy::repo::tauri_message_hook::message_hook,
        legacy::cherry_apply::tauri_cherry_apply_status::cherry_apply_status,
        legacy::cherry_apply::tauri_cherry_apply::cherry_apply,
        legacy::virtual_branches::tauri_create_virtual_branch::create_virtual_branch,
        legacy::virtual_branches::tauri_delete_local_branch::delete_local_branch,
        legacy::virtual_branches::tauri_get_base_branch_data::get_base_branch_data,
        legacy::virtual_branches::tauri_set_base_branch::set_base_branch,
        legacy::virtual_branches::tauri_switch_back_to_workspace::switch_back_to_workspace,
        legacy::virtual_branches::tauri_push_base_branch::push_base_branch,
        legacy::virtual_branches::tauri_integrate_upstream_commits::integrate_upstream_commits,
        legacy::virtual_branches::tauri_get_initial_integration_steps_for_branch::get_initial_integration_steps_for_branch,
        legacy::virtual_branches::tauri_update_stack_order::update_stack_order,
        legacy::virtual_branches::tauri_unapply_stack::unapply_stack,
        legacy::virtual_branches::tauri_create_virtual_branch_from_branch::create_virtual_branch_from_branch,
        legacy::virtual_branches::tauri_list_branches::list_branches,
        legacy::virtual_branches::tauri_get_branch_listing_details::get_branch_listing_details,
        legacy::virtual_branches::tauri_integrate_branch_with_steps::integrate_branch_with_steps,
        legacy::virtual_branches::tauri_fetch_from_remotes::fetch_from_remotes,
        legacy::virtual_branches::tauri_normalize_branch_name::normalize_branch_name,
        legacy::virtual_branches::tauri_upstream_integration_statuses::upstream_integration_statuses,
        legacy::virtual_branches::tauri_integrate_upstream::integrate_upstream,
        legacy::virtual_branches::tauri_resolve_upstream_integration::resolve_upstream_integration,
        branch::tauri_get_initial_branch_integration::get_initial_branch_integration,
        branch::tauri_apply_branch_integration::apply_branch_integration,
        legacy::stack::tauri_create_reference::create_reference,
        legacy::stack::tauri_create_branch::create_branch,
        legacy::stack::tauri_remove_branch::remove_branch,
        legacy::stack::tauri_update_branch_name::update_branch_name,
        legacy::stack::tauri_update_branch_pr_number::update_branch_pr_number,
        legacy::stack::tauri_push_stack::push_stack,
        legacy::secret::tauri_secret_get_global::secret_get_global,
        legacy::secret::tauri_secret_set_global::secret_set_global,
        legacy::secret::tauri_secret_delete_global::secret_delete_global,
        legacy::oplog::tauri_list_snapshots::list_snapshots,
        legacy::oplog::tauri_create_snapshot::create_snapshot,
        legacy::oplog::tauri_restore_snapshot::restore_snapshot,
        legacy::oplog::tauri_snapshot_diff::snapshot_diff,
        legacy::config::tauri_get_gb_config::get_gb_config,
        legacy::config::tauri_set_gb_config::set_gb_config,
        legacy::config::tauri_store_author_globally_if_unset::store_author_globally_if_unset,
        legacy::config::tauri_get_author_info::get_author_info,
        legacy::remotes::tauri_list_remotes::list_remotes,
        legacy::remotes::tauri_add_remote::add_remote,
        legacy::modes::tauri_operating_mode::operating_mode,
        legacy::modes::tauri_head_sha::head_sha,
        legacy::modes::tauri_enter_edit_mode::enter_edit_mode,
        legacy::modes::tauri_save_edit_and_return_to_workspace::save_edit_and_return_to_workspace,
        legacy::modes::tauri_abort_edit_and_return_to_workspace::abort_edit_and_return_to_workspace,
        legacy::modes::tauri_edit_initial_index_state::edit_initial_index_state,
        legacy::modes::tauri_edit_changes_from_initial::edit_changes_from_initial,
        open::tauri_open_url::open_url,
        open::tauri_open_in_terminal::open_in_terminal,
        open::tauri_show_in_finder::show_in_finder,
        open::terminal::tauri_get_terminal_options_for_platform::get_terminal_options_for_platform,
        open::terminal::tauri_get_recommended_terminal_for_platform::get_recommended_terminal_for_platform,
        legacy::forge::tauri_pr_templates::pr_templates,
        legacy::forge::tauri_pr_template::pr_template,
        legacy::forge::tauri_forge_provider::forge_provider,
        legacy::forge::tauri_forge_info::forge_info,
        legacy::forge::tauri_forge_compare_branch_url::forge_compare_branch_url,
        legacy::forge::tauri_list_reviews::list_reviews,
        legacy::forge::tauri_get_review::get_review,
        legacy::forge::tauri_get_review_merge_status::get_review_merge_status,
        legacy::forge::tauri_get_review_base_repo_url::get_review_base_repo_url,
        legacy::forge::tauri_get_repo_info::get_repo_info,
        legacy::forge::tauri_update_review::update_review,
        legacy::forge::tauri_list_ci_checks::list_ci_checks,
        legacy::forge::tauri_publish_review::publish_review,
        legacy::forge::tauri_merge_review::merge_review,
        legacy::forge::tauri_set_review_auto_merge::set_review_auto_merge,
        legacy::forge::tauri_set_review_draftiness::set_review_draftiness,
        legacy::forge::tauri_update_review_footers::update_review_footers,
        legacy::cli::tauri_install_cli::install_cli,
        legacy::cli::tauri_cli_path::cli_path,
        legacy::rules::tauri_create_workspace_rule::create_workspace_rule,
        legacy::rules::tauri_delete_workspace_rule::delete_workspace_rule,
        legacy::rules::tauri_update_workspace_rule::update_workspace_rule,
        legacy::rules::tauri_list_workspace_rules::list_workspace_rules,
        legacy::workspace::tauri_head_info::head_info,
        legacy::workspace::tauri_branch_details::branch_details,
        legacy::workspace::tauri_discard_worktree_changes::discard_worktree_changes,
        legacy::workspace::tauri_stash_into_branch::stash_into_branch,
        legacy::workspace::tauri_canned_branch_name::canned_branch_name,
        legacy::workspace::tauri_target_commits::target_commits,
        legacy::workspace::tauri_workspace_branch_and_ancestors_push::workspace_branch_and_ancestors_push,
        legacy::absorb::tauri_absorb::absorb,
        legacy::absorb::tauri_absorption_plan::absorption_plan,
        diff::tauri_changes_in_worktree::changes_in_worktree,
        diff::tauri_tree_change_diffs::tree_change_diffs,
        diff::tauri_assign_hunk::assign_hunk,
        #[cfg(unix)]
        legacy::workspace::tauri_show_graph_svg::show_graph_svg,
        $($governance_command),*,
        action::list_actions,
        action::handle_changes,
        action::list_workflows,
        askpass::submit_prompt_response,
        menu::menu_item_set_enabled,
        projects::list_projects,
        projects::server_capabilities,
        projects::set_project_active,
        projects::open_project_in_window,
        zip::get_logs_archive_path,
        zip::get_project_archive_path,
        zip::get_anonymous_graph_path,
        settings::get_app_settings,
        settings::update_onboarding_complete,
        settings::update_telemetry,
        settings::update_feature_flags,
        settings::update_telemetry_distinct_id,
        settings::update_fetch,
        settings::update_reviews,
        settings::update_ui,
        settings::update_irc,
        // Debug-only - not for production!
        #[cfg(debug_assertions)]
        env::env_vars,
        #[cfg(feature = "irc")]
        irc::irc_connect,
        #[cfg(feature = "irc")]
        irc::irc_disconnect,
        #[cfg(feature = "irc")]
        irc::irc_state,
        #[cfg(feature = "irc")]
        irc::irc_wait_ready,
        #[cfg(feature = "irc")]
        irc::irc_join,
        #[cfg(feature = "irc")]
        irc::irc_part,
        #[cfg(feature = "irc")]
        irc::irc_auto_join,
        #[cfg(feature = "irc")]
        irc::irc_auto_leave,
        #[cfg(feature = "irc")]
        irc::irc_send_message,
        #[cfg(feature = "irc")]
        irc::irc_send_message_with_data,
        #[cfg(feature = "irc")]
        irc::irc_send_raw,
        #[cfg(feature = "irc")]
        irc::irc_send_typing,
        #[cfg(feature = "irc")]
        irc::irc_send_reaction,
        #[cfg(feature = "irc")]
        irc::irc_remove_reaction,
        #[cfg(feature = "irc")]
        irc::irc_redact_message,
        #[cfg(feature = "irc")]
        irc::irc_list_connections,
        #[cfg(feature = "irc")]
        irc::irc_exists,
        #[cfg(feature = "irc")]
        irc::irc_nick,
        #[cfg(feature = "irc")]
        irc::irc_request_history,
        #[cfg(feature = "irc")]
        irc::irc_request_history_before,
        #[cfg(feature = "irc")]
        irc::irc_messages,
        #[cfg(feature = "irc")]
        irc::irc_channels,
        #[cfg(feature = "irc")]
        irc::irc_users,
        #[cfg(feature = "irc")]
        irc::irc_mark_read,
        #[cfg(feature = "irc")]
        irc::irc_clear_messages,
        #[cfg(feature = "irc")]
        irc::irc_get_all_commit_reactions,
        #[cfg(feature = "irc")]
        irc::irc_get_all_message_reactions,
        #[cfg(feature = "irc")]
        irc::irc_get_file_message_reactions,
        #[cfg(feature = "irc")]
        irc::irc_get_working_files,
        #[cfg(feature = "irc")]
        irc::irc_start_working_files_broadcast,
        #[cfg(feature = "irc")]
        irc::irc_stop_working_files_broadcast,
        commit::reword::tauri_commit_reword::commit_reword,
        commit::insert_blank::tauri_commit_insert_blank::commit_insert_blank,
        commit::create::tauri_commit_create::commit_create,
        commit::amend::tauri_commit_amend::commit_amend,
        commit::move_commit::tauri_commit_move::commit_move,
        commit::move_changes::tauri_commit_move_changes_between::commit_move_changes_between,
        commit::squash::tauri_commit_squash::commit_squash,
        commit::uncommit::tauri_commit_uncommit_changes::commit_uncommit_changes,
        commit::uncommit::tauri_commit_uncommit::commit_uncommit,
        workspace::tauri_workspace_integrate_upstream::workspace_integrate_upstream,
        platform::tauri_build_type::build_type,
            ]
        };
    }
    gitbutler_governance_command_rows!(full_invoke_handler)
}
