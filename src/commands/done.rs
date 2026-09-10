use crate::git::Repo;
use anyhow::Result;

pub fn run() -> Result<()> {
    let _repo = Repo::discover()?;
    println!("dev done: not implemented yet");
    Ok(())
}
