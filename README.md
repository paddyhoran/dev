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
format. It takes no arguments — everything is gathered interactively:

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
a picker for you to pick from.

When you initially do this there will be no conflicts but as you start to sync commits from 
feature branches in some conflicts will be created.  If conflicts exist they will be 
indicated in the picker so you can continue to merge commits without conflicts as much as
possible.

If you merge a commit that conflicts with `main` then you will be asked to resolve these 
conflicts using `$EDITOR`.

Each time you successfully run the `sync` command `main` will be pushed to update the remote.

> `sync` will only work from `main`

### Technical Approach

For the commit on the feature branch selected by the user that hasn't already been merged 
into `main`, this will:

1. `git cherry-pick` the commit onto `main`.
2. Run `cog bump --auto` (same version-sync logic as `just c` does today: sync
   `Cargo.toml`, amend, re-tag).
3. Push the commit and the new tag to `main`.

It processes commits **oldest first, one at a time**.

**"Already merged" is tracked by content, not by commit hash.** Cherry-picking creates a
new commit (new SHA, same diff and message) on `main`. `sync` uses
`git cherry -v main <name>`, which compares by patch content rather than SHA, so it
correctly skips commits already merged in a previous `sync` run. This means:

- You can run `sync` as many times as you like over the life of a feature —
  each run only picks up what's new since last time.
- You don't need any separate bookkeeping (no "last merged commit" file to maintain).

**If a cherry-pick conflicts**, `dev` stops and tells you to resolve the conflict and opens 
$EDITOR, once done it runs `git cherry-pick --continue`and resumes exactly where it left off.

## 4 - Updating feature branches

Feature branches will continue independently to `main` and when `sync` is run it recognises 
previous commits on the feature branch that have already been merged.

However, you may want to update your feature branch to reflect the current `main`.  You do this
by running `dev update` on a feature branch.  This rebases the branch onto the latest `main`
and, once done, force-pushes the feature branch so the remote matches.

## 5 - Bumping version numbers

`dev` uses the [Cocogitto](https://docs.cocogitto.io) tool to automatically manage 
versioning.

You can use `dev bump`, this will:
 - update the changelog and commit it
 - bump the version number
 - create a version tag
 - push to remote (including tags)

## 6 - Finishing a feature branch

When a feature is fully synced into `main` and you're ready to wrap it up, run `dev done`
(alias `dev d`). A picker will be displayed showing all current features. When you select
a feature `dev` will:
 - run a version of `sync` repeatedly that focuses on the feature branch only
 - once all changes are added `main`:
   - `cd` back to the main folder
   - removed the branch locally and in the remote
   - remove the git worktree

