use anyhow::{Result, bail};

/// Render the initial contents shown in `$EDITOR`: a pre-filled
/// `type(scope): ` subject prefix, a blank body area, and `#`-comment
/// instructions at the bottom (stripped later, same convention as `git
/// commit`'s own template). The ticket footer is shown only as a comment
/// hint here — the real footer line is appended programmatically in
/// `finalize`, so it can't be accidentally deleted or malformed by hand.
pub fn render(commit_type: &str, scope: &str, ticket_footer: Option<&str>) -> String {
    let mut out = format!("{commit_type}({scope}): \n\n");
    if let Some(footer) = ticket_footer {
        out.push_str(&format!("# This commit will add the trailer: {footer}\n"));
    }
    out.push_str("#\n");
    out.push_str("# Write the subject on the first line above, then the body here.\n");
    out.push_str(&format!("# type:  {commit_type}\n"));
    out.push_str(&format!("# scope: {scope}\n"));
    out.push_str(
        "# Lines starting with '#' are ignored, and an empty subject aborts the commit.\n",
    );
    out
}

/// Strip `#`-comment lines from the edited template, validate the subject
/// (the text typed after the `type(scope): ` prefix) is non-empty, and
/// append the real ticket footer, if any.
pub fn finalize(
    commit_type: &str,
    scope: &str,
    edited: &str,
    ticket_footer: Option<&str>,
) -> Result<String> {
    // Kept unjoined (not trimmed as one blob) so the first line's trailing
    // space — part of the `type(scope): ` prefix — survives for the
    // prefix-match below; joining-then-trimming would eat it and make an
    // untouched template look like it had a real subject.
    let stripped_lines: Vec<&str> = edited
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect();

    let first_line = stripped_lines.first().copied().unwrap_or("");
    let rest = if stripped_lines.len() > 1 {
        stripped_lines[1..].join("\n")
    } else {
        String::new()
    };

    let prefix = format!("{commit_type}({scope}): ");
    let subject = first_line
        .strip_prefix(&prefix)
        .unwrap_or(first_line)
        .trim();
    if subject.is_empty() {
        bail!("commit subject is empty — aborting");
    }

    let mut message = first_line.trim_end().to_string();
    let body = rest.trim();
    if !body.is_empty() {
        message.push_str("\n\n");
        message.push_str(body);
    }
    if let Some(footer) = ticket_footer {
        message.push_str("\n\n");
        message.push_str(footer);
    }

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_comments_and_keeps_subject_and_body() {
        let edited = "feat(cli): add a thing\n\nBody line.\n# comment\n";
        let msg = finalize("feat", "cli", edited, None).unwrap();
        assert_eq!(msg, "feat(cli): add a thing\n\nBody line.");
    }

    #[test]
    fn rejects_unedited_subject() {
        let edited = "feat(cli): \n\n# comment\n";
        assert!(finalize("feat", "cli", edited, None).is_err());
    }

    #[test]
    fn appends_ticket_footer_after_body() {
        let edited = "feat(cli): add a thing\n";
        let msg = finalize("feat", "cli", edited, Some("Fixes: #42")).unwrap();
        assert_eq!(msg, "feat(cli): add a thing\n\nFixes: #42");
    }
}
