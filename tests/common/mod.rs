// Each integration test file compiles this module separately and only ever
// uses a subset of it, so dead_code would false-positive per binary.
#![allow(dead_code)]

use dev::picker::Prompter;
use std::cell::RefCell;
use std::collections::VecDeque;
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

    pub fn write_config(&self, contents: &str) {
        std::fs::write(self.work.join(".dev-config.toml"), contents).expect("write config");
    }

    /// Run an arbitrary git command in the fixture's working repo, e.g. to
    /// stage a change before exercising `dev commit`.
    pub fn git(&self, args: &[&str]) {
        run_git(&self.work, args);
    }

    /// Write an executable shell script into the fixture and return its
    /// path, for use as the `editor` argument to `dev::editor::edit_template`.
    /// `body` receives the commit-message file's path as `$1`.
    pub fn write_fake_editor(&self, body: &str) -> PathBuf {
        let path = self.work.join("fake_editor.sh");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write fake editor");
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
        std::fs::set_permissions(&path, perms).unwrap();
        path
    }

    pub fn repo(&self) -> dev::git::Repo {
        dev::git::Repo::discover_from(&self.work).expect("discover fixture repo")
    }
}

/// A `Prompter` that returns a fixed issue number and a queue of canned
/// `select()` answers, so `dev commit`'s flow can run headlessly in tests —
/// `dialoguer::Select` itself needs a real TTY and can't be driven this way.
pub struct ScriptedPrompter {
    issue_number: Option<u64>,
    selections: RefCell<VecDeque<String>>,
    /// `(label, options)` for every `select()` call, in order — lets tests
    /// assert the picker was only ever offered the configured values.
    pub seen: RefCell<Vec<(String, Vec<String>)>>,
}

impl ScriptedPrompter {
    pub fn new(issue_number: Option<u64>, selections: &[&str]) -> Self {
        Self {
            issue_number,
            selections: RefCell::new(selections.iter().map(|s| s.to_string()).collect()),
            seen: RefCell::new(Vec::new()),
        }
    }
}

impl Prompter for ScriptedPrompter {
    fn ask_issue_number(&self) -> anyhow::Result<Option<u64>> {
        Ok(self.issue_number)
    }

    fn select(&self, label: &str, options: &[String]) -> anyhow::Result<String> {
        self.seen
            .borrow_mut()
            .push((label.to_string(), options.to_vec()));
        self.selections
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("ScriptedPrompter: no scripted answer left"))
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
