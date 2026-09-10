use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevError {
    #[error("not inside a git repository")]
    NotInRepo,

    #[error("`{command}` must be run from the main branch (currently on `{branch}`)")]
    NotOnMainBranch {
        command: &'static str,
        branch: String,
    },

    #[error("`{command}` must be run from a feature branch, not `{branch}`")]
    NotOnFeatureBranch {
        command: &'static str,
        branch: String,
    },

    #[error("git {args} failed:\n{stderr}")]
    GitCommandFailed { args: String, stderr: String },
}
