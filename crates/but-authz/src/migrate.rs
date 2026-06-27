//! Text-preserving migration of the legacy `permissions.toml` wire format to the
//! `agents.toml` wire format.
//!
//! `but-authz` owns both governance wire formats, so it also owns the transform
//! between them. The only wire-level difference is the array-of-tables header:
//! `[[principal]]` becomes `[[agent]]`. Everything else — body bytes, comments,
//! blank lines, indentation, and line endings — is preserved verbatim so the
//! migration is a faithful rename rather than a lossy re-serialization.
//!
//! Both the `but agent migrate` CLI verb and the IDENT-013 round-trip test call
//! this one function, so a comment-stripping regression is caught in one place.

/// Rewrite every `[[principal]]` array-of-tables header in `contents` to
/// `[[agent]]`, preserving all other bytes exactly.
///
/// Only genuine table headers are rewritten: a `[[principal]]` token is
/// converted when it begins a line (after optional leading spaces or tabs) and
/// is followed only by whitespace or a `#` comment. Occurrences inside string
/// values or comment bodies are left untouched. Comments, blank lines,
/// indentation, and `\r\n`/`\n` line endings round-trip unchanged.
///
/// ```
/// let permissions = "# leading comment\n[[principal]]\nid = \"dev\"\n";
/// let agents = but_authz::rewrite_principals_to_agents(permissions);
/// assert_eq!(agents, "# leading comment\n[[agent]]\nid = \"dev\"\n");
/// ```
pub fn rewrite_principals_to_agents(contents: &str) -> String {
    let mut rewritten = String::with_capacity(contents.len());
    for line in contents.split_inclusive('\n') {
        let (line_without_lf, lf) = match line.strip_suffix('\n') {
            Some(line_without_lf) => (line_without_lf, "\n"),
            None => (line, ""),
        };
        let (line_without_eol, cr) = match line_without_lf.strip_suffix('\r') {
            Some(line_without_eol) => (line_without_eol, "\r"),
            None => (line_without_lf, ""),
        };
        let trimmed = line_without_eol.trim_start_matches([' ', '\t']);
        let prefix_len = line_without_eol.len() - trimmed.len();

        if let Some(suffix) = trimmed.strip_prefix("[[principal]]")
            && is_table_header_suffix(suffix)
        {
            rewritten.push_str(&line_without_eol[..prefix_len]);
            rewritten.push_str("[[agent]]");
            rewritten.push_str(suffix);
            rewritten.push_str(cr);
            rewritten.push_str(lf);
        } else {
            rewritten.push_str(line);
        }
    }
    rewritten
}

/// A `[[principal]]` token is a table header only when nothing but whitespace or
/// a `#` comment follows it on the line.
fn is_table_header_suffix(suffix: &str) -> bool {
    let suffix = suffix.trim_start_matches([' ', '\t']);
    suffix.is_empty() || suffix.starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::rewrite_principals_to_agents;

    #[test]
    fn rewrites_bare_table_header() {
        assert_eq!(
            rewrite_principals_to_agents("[[principal]]\nid = \"dev\"\n"),
            "[[agent]]\nid = \"dev\"\n"
        );
    }

    #[test]
    fn preserves_comments_blank_lines_and_inline_comment() {
        let permissions =
            "# leading\n[[principal]]\nid = \"dev\"\n\n[[principal]]  # trailing\nid = \"ro\"\n";
        let expected = "# leading\n[[agent]]\nid = \"dev\"\n\n[[agent]]  # trailing\nid = \"ro\"\n";
        assert_eq!(
            rewrite_principals_to_agents(permissions),
            expected,
            "comments, blank lines, and inline header comments must survive"
        );
    }

    #[test]
    fn preserves_leading_indentation_and_crlf() {
        assert_eq!(
            rewrite_principals_to_agents("\t[[principal]]\r\nid = \"dev\"\r\n"),
            "\t[[agent]]\r\nid = \"dev\"\r\n",
            "indentation and CRLF line endings must round-trip"
        );
    }

    #[test]
    fn ignores_principal_token_in_string_value() {
        let contents = "[[agent]]\nnote = \"see [[principal]] docs\"\n";
        assert_eq!(
            rewrite_principals_to_agents(contents),
            contents,
            "a [[principal]] token inside a value must not be rewritten"
        );
    }
}
