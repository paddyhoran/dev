use crate::error::DevError;
use crate::git::Repo;
use anyhow::Result;

pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    if repo.is_on_main()? {
        return Err(DevError::NotOnFeatureBranch {
            command: "update",
            branch: repo.current_branch()?,
        }
        .into());
    }
    println!("dev update: not implemented yet");
    Ok(())
}
