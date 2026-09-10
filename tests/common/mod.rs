use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

/// A throwaway `origin` (bare) + cloned working repo, built fresh per test.
/// Kept alive for the test's duration by holding onto the `TempDir`.
pub struct Fixture {
    _tmp: TempDir,
    pub work: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let tmp = TempDir::new().expect("create tempdir");
        let origin = tmp.path().join("origin.git");
        let work = tmp.path().join("work");

        run_git(
            tmp.path(),
            &["init", "--bare", "-b", "main", path_str(&origin)],
        );
        run_git(tmp.path(), &["clone", path_str(&origin), path_str(&work)]);
        run_git(&work, &["config", "user.email", "test@example.com"]);
        run_git(&work, &["config", "user.name", "Test"]);
        std::fs::write(work.join("README.md"), "test repo\n").expect("write README");
        run_git(&work, &["add", "README.md"]);
        run_git(&work, &["commit", "-m", "chore: initial commit"]);
        run_git(&work, &["push", "-u", "origin", "main"]);

        Self { _tmp: tmp, work }
    }

    /// Run the built `dev` binary with the given args, cwd defaulting to the
    /// main working repo.
    pub fn dev(&self, args: &[&str]) -> Output {
        self.dev_in(&self.work, args)
    }

    pub fn dev_in(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dev"))
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("run dev binary")
    }
}

pub fn path_str(p: &Path) -> &str {
    p.to_str().expect("path must be valid UTF-8")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn `git {args:?}`: {e}"));
    assert!(
        output.status.success(),
        "`git {args:?}` failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
