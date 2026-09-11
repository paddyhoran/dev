use anyhow::Result;
use dialoguer::{Confirm, Input, MultiSelect, Select};

/// Abstraction over `dev`'s interactive prompts. `dialoguer`'s widgets read
/// raw key events straight from a TTY, so they can't be driven by piping
/// stdin to a subprocess — tests supply a scripted implementation instead of
/// exercising `DialoguerPrompter` against a real terminal.
pub trait Prompter {
    /// Ask for an optional issue number. `None` means "this change does not
    /// close an issue".
    fn ask_issue_number(&self) -> Result<Option<u64>>;

    /// Offer `options` under `label` and return the one the user picked.
    fn select(&self, label: &str, options: &[String]) -> Result<String>;

    /// Offer `options` as a checklist under `label` and return the indices
    /// the user selected, in `options` order. An empty result (nothing
    /// selected, confirmed as-is) is a valid "done for now" signal — callers
    /// treat it that way rather than as an error.
    fn select_many(&self, label: &str, options: &[String]) -> Result<Vec<usize>>;

    /// A yes/no question, e.g. "run `dev bump` now?".
    fn confirm(&self, message: &str) -> Result<bool>;
}

pub struct DialoguerPrompter;

impl Prompter for DialoguerPrompter {
    fn ask_issue_number(&self) -> Result<Option<u64>> {
        let answer: String = Input::new()
            .with_prompt("Issue number (leave blank if none)")
            .allow_empty(true)
            .validate_with(|input: &String| parse_issue_number(input).map(|_| ()))
            .interact_text()?;

        Ok(parse_issue_number(&answer).expect("validated by validate_with"))
    }

    fn select(&self, label: &str, options: &[String]) -> Result<String> {
        let idx = Select::new()
            .with_prompt(label)
            .items(options)
            .default(0)
            .interact()?;
        Ok(options[idx].clone())
    }

    fn select_many(&self, label: &str, options: &[String]) -> Result<Vec<usize>> {
        Ok(MultiSelect::new()
            .with_prompt(label)
            .items(options)
            .interact()?)
    }

    fn confirm(&self, message: &str) -> Result<bool> {
        Ok(Confirm::new()
            .with_prompt(message)
            .default(false)
            .interact()?)
    }
}

/// Blank input means "no issue"; anything else must parse as a whole
/// non-negative number. Pulled out of the dialoguer validator closure so it
/// can be unit-tested without a TTY.
fn parse_issue_number(input: &str) -> Result<Option<u64>, &'static str> {
    match input.trim() {
        "" => Ok(None),
        trimmed => trimmed
            .parse::<u64>()
            .map(Some)
            .map_err(|_| "enter a whole number, or leave blank"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_input_means_no_issue() {
        assert_eq!(parse_issue_number("   "), Ok(None));
    }

    #[test]
    fn parses_a_valid_integer() {
        assert_eq!(parse_issue_number("42"), Ok(Some(42)));
    }

    #[test]
    fn rejects_non_integers() {
        assert!(parse_issue_number("abc").is_err());
        assert!(parse_issue_number("-1").is_err());
        assert!(parse_issue_number("12.5").is_err());
    }
}
