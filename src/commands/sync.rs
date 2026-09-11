use crate::error::DevError;
use crate::git::Repo;
use anyhow::Result;

pub fn run(repo: &Repo) -> Result<()> {
    if !repo.is_on_main()? {
        return Err(DevError::NotOnMainBranch {
            command: "sync",
            branch: repo.current_branch()?,
        }
        .into());
    }
    println!("dev sync: not implemented yet");
    Ok(())
}
