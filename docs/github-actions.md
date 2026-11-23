# GitHub Actions maintenance

This repo locks every GitHub Action its workflows use, so CI runs only code that someone reviewed in
a PR. This guide covers what that means, and what to do when you add an action, review a Dependabot
PR, or see the lockfile check fail.

## Why

Workflows run third-party actions with access to the repo's token, and release workflows also have
publish credentials. A tag like `v4` is a pointer that the action's owner, or anyone who takes over
their account, can move to different code. `.github/workflows/actions.lock` records the exact commit
behind every action, including the actions those actions call. GitHub runs those commits and nothing
else, and any change to them shows up as a diff you can review.

The lockfile doesn't cover:

- calls to reusable workflows in other repos (`jobs.<id>.uses`). Pin those to a full commit SHA with
  a `# vX.Y.Z` comment. The local call to `ci.yml` runs from the same commit, and the lockfile
  covers the actions inside it.
- container images. Pin those by `@sha256:` digest.
- anything a step downloads while it runs, including the lockfile extension. Jobs that use
  `fallback: none` accept only tools with a checksum in the locked install action.
- the runner image.

## How CI uses it

The `lockfile` job is part of `.github/workflows/ci.yml`. The release workflow calls `ci.yml` and
publishes only after all of its jobs pass, so a lockfile failure blocks publication. `main` has no
required checks, so confirm that CI passed before merging a PR.

The `lockfile` job runs `gh actions-lock --verify`. It fails when the lockfile doesn't match the
workflows, or when a locked commit no longer checks out upstream.

## Setup

Install the extension with the command in [Contributing](../CONTRIBUTING.md#setup). It matches the
version in CI's `lockfile` job, because different versions may write different lockfiles. Dependabot
does not update this version or the tools listed in `ci.yml`. When updating one, read its release
notes and change `ci.yml` and the setup guide together.

## Common tasks

### Add or change an action

1. Edit the `uses:` line. Use an exact release tag (`owner/action@v1.2.3`) when the action publishes
   one. Don't use major-only tags like `@v2`.
2. Run `gh actions-lock`. It writes the new entry and doesn't change the existing ones.
3. Commit the workflow and `actions.lock` together. In the PR, say why you chose that ref.

### Review a Dependabot PR

Dependabot updates the workflow and `actions.lock` in the same PR. Before merging, check:

- the release notes for the new version.
- that only the commit for the action being bumped changed in `actions.lock`.
- that `owner_id` and `repo_id` didn't change. If they did, the repository behind that name is a
  different one, so investigate before merging.

### The lockfile check failed

Run `gh actions-lock --verify` locally. Each finding links to its fix.

| Finding                                              | Do                                                                                      |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `not-pinned`, `stale`, `ref-changed`, `repo-renamed` | Run `gh actions-lock` and commit.                                                       |
| `tag-commit-mismatch`, `incomplete-lockfile`         | Read the upstream diff, then run `--relock` on the affected workflow.                   |
| `reachability-unknown`, `ancestry-unknown`           | GitHub didn't respond. Re-run the job.                                                  |
| `unreachable-pin`, `misleading-sha`, `repo-replaced` | **Stop.** Either upstream history was rewritten or this is an attack. Ask a maintainer. |

Warnings such as `ref-moved` don't fail the check.

## Never

- Edit `actions.lock` by hand.
- Delete it to make an error go away. Regenerating it from scratch accepts whatever every tag points
  at today, and the record of what you reviewed is lost.
- Run `--accept-moved` in CI or a script. Run it only after a person has checked why the commit
  moved.

To undo a bad lockfile change, restore the file from the default branch and run `gh actions-lock`
again:

```sh
jj restore --from 'trunk()' .github/workflows/actions.lock     # jj
git checkout origin/HEAD -- .github/workflows/actions.lock     # git
```

## More

- [gh-actions-lock](https://github.com/github/gh-actions-lock) and its
  [troubleshooting guide](https://github.com/github/gh-actions-lock/blob/main/docs/troubleshooting.md)
- [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
