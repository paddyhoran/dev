# Personal Development Workflow

This repository contains the implemenation of `dev` a CLI tool that implements my personal 
workflow system that I use in the new "agentic coding" era when I am operating in "solo" 
mode (i.e. my collaborators are agents not humans).  It is evolving all the time...

## General Concept

I am not yet at the point of running a software factory and I'm not sure I ever will.  I
**still review and hold myself accountable** for all code merged.

To this end, I operate a funnel of sorts where "features" live on branches can and can be 
worked on in parallel.  Within each feature branch I use conventional commits and review
all code, I hold myself to the same standards as `main` on these branches.

I then use a separate process to pull in updates to all my feature branches into `main`.
As this process is done the changelog, versions and tags are all updated and pushed.  So 
"features" are not done until funnelled through the process of added them to main.

## Commands at a glance

Every command also has a single-letter alias, so `dev d` works the same as `dev done`:

| Command      | Alias  | Purpose                                    |
|--------------|--------|---------------------------------------------|
| `dev new`    | `dev n`| Start a new feature branch + worktree      |
| `dev commit` | `dev c`| Create a conventional commit interactively |
| `dev sync`   | `dev s`| Pull feature commits into `main`           |
| `dev update` | `dev u`| Rebase a feature branch onto latest `main` |
| `dev bump`   | `dev b`| Bump version, update changelog, tag, push  |
| `dev done`   | `dev d`| Finish and remove a fully-merged feature   |

## 1 - Creating a new feature branch

The command `dev new <name>` will:

1. Fetch the latest `main`.
2. Create a new branch `<name>` off `origin/main`.
3. Create a new worktree for it at `.worktrees/<name>`.
4. Print the path so you can `cd` into it.

You now have an isolated working directory and branch. Nothing here affects `main` or any
other feature until you explicitly merge it back.

I use WezTerm workspaces to switch between feature branches.

## 2 - Commiting Changes (feature or `main`)

The command `dev commit` (alias `dev c`) creates a new commit on the current branch,
matching the [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)
format. It takes no arguments — everything is gathered interactively.

Before asking anything, `dev commit` checks that there's nothing unstaged or
untracked sitting in the working tree — stage exactly what you mean to commit
first (`git add`), or it'll ask you to sort that out before continuing.

Then:

1. **Issue number** — you're asked for an issue number, or you can leave it blank if
   this change doesn't close one. If you enter something, it must be a plain integer;
   anything else (letters, a negative number, decimals) is rejected and you're asked
   again.
2. **Type** — you pick from a list of commit types (`feat`, `fix`, `chore`, ...).
3. **Scope** — you pick from a list of scopes.

Both lists come from a project-specific `.dev-config.toml` file, so only options
valid for this project are ever offered.

Once all three answers are given, `dev` opens `$EDITOR` with a template pre-filled
from your answers (including a link to the closed ticket, if you gave an issue
number) so you can write the commit body.

## 3 - Pulling changes into `main`

Once you have created some feature branches and started to commit changes to them you will 
need to start to move these changes to `main`.

This is done by `dev sync`, this will scan all your feature branches and identify the next
commit for each feature branch.  The commits available to be applied will be presented in 
a checklist for you to pick from — pick as many as you like, they're applied oldest-first
per branch. Leave everything unselected and confirm to stop for now.

Each commit you sync is cherry-picked onto `main` and **pushed immediately**, one at a
time. `sync` does **not** bump the version or create a tag as it goes — once you're done
syncing (either you've applied everything available, or you chose to stop), it asks once:
"Run `dev bump` now?" Answering yes runs `dev bump` exactly as if you'd typed it yourself.
Deferring it this way means one clean version bump covering everything you just synced,
instead of a version bump interleaved after every single commit — and if you stop syncing
without bumping, nothing is lost: every commit synced so far is already pushed, and
`dev bump` can be run any time later.

> `sync` will only work from `main`

**"Already merged" is tracked by content, not by commit hash.** Cherry-picking creates a
new commit (new SHA, same diff and message) on `main`. `sync` uses
`git cherry -v main <name>`, which compares by patch content rather than SHA, so it
correctly skips commits already merged in a previous `sync` run. This means:

- You can run `sync` as many times as you like over the life of a feature —
  each run only picks up what's new since last time.
- You don't need any separate bookkeeping (no "last merged commit" file to maintain).
- Because it compares actual diffs, a commit you hand-resolve through a real conflict
  may still show up as a candidate again afterward — your resolution rarely produces
  a byte-identical patch to the original. That's expected, not a bug; just skip it if
  it reappears with nothing new to offer.

**If a cherry-pick conflicts**, `dev` opens `$EDITOR` on the repo so you can resolve
the conflict markers yourself. Once you close the editor, it checks whether the
conflicted files still contain markers; if not, it stages them and continues the
cherry-pick automatically. If markers remain, it asks whether to open the editor
again or give up — giving up aborts the cherry-pick and stops `sync` for this run
(anything already synced before that point stays synced; you're still asked about
`dev bump` if so).

### Setup requirement: gitignore `.worktrees/`

Any project using `dev` **must** add `/.worktrees/` to its `.gitignore`. If it isn't
ignored, `git status` reports it as untracked the moment any feature branch exists, which
breaks `dev bump` outright — Cocogitto has its own untracked-files check that `dev` can't
work around (the obvious-looking fix, `--skip-untracked`, actually makes it worse: cog
tries to include the directory and crashes trying to add a nested worktree as a regular
path). `dev commit`'s and `dev bump`'s own checks tolerate `.worktrees/` regardless, but
Cocogitto doesn't, so the gitignore entry is required, not just tidy.

## 4 - Updating feature branches

Feature branches will continue independently to `main` and when `sync` is run it recognises 
previous commits on the feature branch that have already been merged.

However, you may want to update your feature branch to reflect the current `main`.  You do this
by running `dev update` (alias `dev u`) on a feature branch.  This rebases the branch onto the
latest `main` and, once done, force-pushes (safely — `--force-with-lease`, so it refuses if
someone else pushed to your feature branch in the meantime) just that feature branch, never
`main`.

If the rebase conflicts, it's the same experience as a conflicting `sync`: `dev` opens
`$EDITOR` on the repo so you can resolve it, then continues automatically once the conflict
markers are gone, or aborts cleanly if you'd rather give up and try again later.

## 5 - Bumping version numbers

`dev` uses the [Cocogitto](https://docs.cocogitto.io) tool to automatically manage 
versioning. `dev bump` (alias `dev b`) only runs on `main`, and only with a fully
clean working tree — commit or stash everything first.

It will:
 - update the changelog and commit it
 - bump the version number and create a version tag
 - sync the new version into a manifest file, if `.dev-config.toml` has a
   `[version_file]` section (see below) — Cocogitto itself only manages the
   changelog and tag, it doesn't know how to edit a project's manifest
 - push to remote (including the new tag)

If nothing has changed since the last bump, `dev bump` says so and exits cleanly
without pushing anything.

### Syncing a manifest's version field

Since which file holds a project's version (`Cargo.toml`, `package.json`,
`pyproject.toml`, ...) is language-specific, `dev` doesn't hardcode any of them.
Instead, `.dev-config.toml` can declare one:

```toml
[version_file]
path = "Cargo.toml"
pattern = '(?m)^version = "([^"]*)"'
```

`pattern` is a regular expression with exactly one capture group wrapping the
version text — `dev` replaces whatever that group matches with the new version
(a leading `v` is stripped automatically, since git tags often have one but
manifest version fields generally can't). Make sure `pattern` is specific enough
that it can only match the one line you mean — e.g. a plain `version = "..."`
could also match a pinned dependency's version elsewhere in the same file.

## 6 - Finishing a feature branch

When a feature is fully synced into `main` and you're ready to wrap it up, run `dev done`
(alias `dev d`). A picker will be displayed showing all current features. When you select
a feature `dev` will:
 - run a version of `sync` repeatedly that focuses on the feature branch only
 - once all changes are added `main`:
   - `cd` back to the main folder
   - removed the branch locally and in the remote
   - remove the git worktree

