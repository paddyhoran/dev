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
        // Any real project using `dev` needs this — `.worktrees/` must be
        // gitignored so it's never untracked (see `dev::cog::bump_auto`'s
        // doc comment for what goes wrong otherwise).
        std::fs::write(work.join(".gitignore"), "/.worktrees/\n").expect("write .gitignore");
        run_git(&work, &["add", "README.md", ".gitignore"]);
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

    /// Same, but in an arbitrary directory (e.g. a feature worktree under
    /// `.worktrees/`) rather than the main working repo.
    pub fn git_in(&self, cwd: &Path, args: &[&str]) {
        run_git(cwd, args);
    }

    /// Create a feature worktree at `.worktrees/<name>` off the current
    /// `main`, mirroring what `dev new` does, and return its path.
    pub fn add_feature_worktree(&self, name: &str) -> PathBuf {
        let path = self.work.join(".worktrees").join(name);
        run_git(
            &self.work,
            &["worktree", "add", "-b", name, path_str(&path)],
        );
        path
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

/// A `Prompter` driven by canned answers, so `dev`'s interactive flows can
/// run headlessly in tests — `dialoguer`'s widgets need a real TTY and can't
/// be driven by piping stdin to a subprocess.
pub struct ScriptedPrompter {
    issue_number: Option<u64>,
    selections: RefCell<VecDeque<String>>,
    select_many_answers: RefCell<VecDeque<Vec<usize>>>,
    confirm_answers: RefCell<VecDeque<bool>>,
    /// `(label, options)` for every `select()` call, in order — lets tests
    /// assert the picker was only ever offered the configured values.
    pub seen: RefCell<Vec<(String, Vec<String>)>>,
    /// Same, for `select_many()` calls.
    pub seen_many: RefCell<Vec<(String, Vec<String>)>>,
}

impl ScriptedPrompter {
    /// For `dev commit`-style flows: a fixed issue number plus a queue of
    /// canned `select()` answers.
    pub fn new(issue_number: Option<u64>, selections: &[&str]) -> Self {
        Self {
            issue_number,
            selections: RefCell::new(selections.iter().map(|s| s.to_string()).collect()),
            select_many_answers: RefCell::new(VecDeque::new()),
            confirm_answers: RefCell::new(VecDeque::new()),
            seen: RefCell::new(Vec::new()),
            seen_many: RefCell::new(Vec::new()),
        }
    }

    /// For `dev sync`-style flows: one entry in `select_many_answers` per
    /// picker round (indices selected that round), and a queue of yes/no
    /// answers for the end-of-run "run `dev bump` now?" confirm.
    pub fn for_sync(select_many_answers: Vec<Vec<usize>>, confirm_answers: Vec<bool>) -> Self {
        Self {
            issue_number: None,
            selections: RefCell::new(VecDeque::new()),
            select_many_answers: RefCell::new(select_many_answers.into()),
            confirm_answers: RefCell::new(confirm_answers.into()),
            seen: RefCell::new(Vec::new()),
            seen_many: RefCell::new(Vec::new()),
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
            .ok_or_else(|| anyhow::anyhow!("ScriptedPrompter: no scripted select() answer left"))
    }

    fn select_many(&self, label: &str, options: &[String]) -> anyhow::Result<Vec<usize>> {
        self.seen_many
            .borrow_mut()
            .push((label.to_string(), options.to_vec()));
        self.select_many_answers
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| {
                anyhow::anyhow!("ScriptedPrompter: no scripted select_many() answer left")
            })
    }

    fn confirm(&self, _message: &str) -> anyhow::Result<bool> {
        self.confirm_answers
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("ScriptedPrompter: no scripted confirm() answer left"))
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
