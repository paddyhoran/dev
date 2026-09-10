use crate::git::Repo;
use anyhow::Result;

pub fn run(name: &str) -> Result<()> {
    let _repo = Repo::discover()?;
    println!("dev new {name}: not implemented yet");
    Ok(())
}
