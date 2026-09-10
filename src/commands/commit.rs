use crate::git::Repo;
use anyhow::Result;

pub fn run() -> Result<()> {
    let _repo = Repo::discover()?;
    println!("dev commit: not implemented yet");
    Ok(())
}
