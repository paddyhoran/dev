use crate::cog;
use crate::config::Config;
use crate::error::DevError;
use crate::git::Repo;
use crate::version_file;
use anyhow::{Context, Result, bail};

pub fn run(repo: &Repo) -> Result<()> {
    if !repo.is_on_main()? {
        return Err(DevError::NotOnMainBranch {
            command: "bump",
            branch: repo.current_branch()?,
        }
        .into());
    }

    if !repo.is_fully_clean()? {
        bail!("working tree is not clean — commit or stash changes before running `dev bump`");
    }

    let old_tag = current_tag(repo);
    cog::bump_auto(&repo.root)?;
    let new_tag = current_tag(repo);

    if old_tag == new_tag {
        println!("dev bump: nothing to bump");
        return Ok(());
    }
    let new_tag = new_tag.context("`cog bump` reported success but no tag was found")?;

    if let Some(version_file_config) = Config::load(&repo.root)?.version_file {
        version_file::sync(&repo.root, &version_file_config, &new_tag)?;
        repo.git.run(&["add", &version_file_config.path])?;
        repo.git.run(&["commit", "--amend", "--no-edit"])?;
        repo.git.run(&["tag", "-f", &new_tag])?;
    }

    // Push new commit and tag.
    repo.git.run(&["push"])?;
    repo.git.run(&["push", "--force", "origin", &new_tag])?;
    Ok(())
}

fn current_tag(repo: &Repo) -> Option<String> {
    repo.git.run(&["describe", "--tags", "--abbrev=0"]).ok()
}
