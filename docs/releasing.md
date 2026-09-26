# Releasing

The workspace publishes the `detect-terminal` library and `detect-terminal-cli` package at a shared
version. [Release-plz](https://release-plz.dev/) prepares a pull request with version and changelog
updates. Merging that PR into `main` authorizes publication after the release workflow passes CI.
The library owns the shared `v<version>` tag and GitHub release.

## Review the Release PR

1. Review the proposed version against changes to the public API, detection precedence, command
   behavior, CLI arguments, and JSON output. Release-plz checks library API compatibility, but
   behavioral changes still need maintainer review.
1. Check that both packages inherit the workspace version and that the CLI's library dependency and
   lockfile agree. Release-plz updates workspace packages without refreshing unrelated dependencies.
   If the proposed version is wrong, use `release-plz set-version <version>` on the release PR and
   inspect the resulting manifests, lockfile, and changelog.
1. Edit `CHANGELOG.md` into useful release notes: explain behavior changes and any migration steps.
   Generated commit summaries are a starting point. Align the README, CLI help, and Rustdoc with
   what ships.
1. Complete the validation below and wait for CI on the latest PR revision before merging. The
   workflow explicitly dispatches CI on release PR branches because they are created with
   `GITHUB_TOKEN`.

Use current stable Cargo for packaging. The minimum supported compiler is a separate compatibility
requirement; see [Contributing](../CONTRIBUTING.md#setup) for the MSRV policy and tool setup.

## Validate the Candidate

Run these checks from the workspace root:

```sh
just check
just msrv
just ci-check
just zizmor
just deny
just package
cargo publish --workspace --dry-run --locked
```

Provide `GH_TOKEN` when running zizmor locally to include its online audits. CI supplies a read-only
GitHub token. Wait for every CI job to pass on the release revision, including native Linux, macOS,
and Windows tests and the MSRV job. A local run establishes only the platforms it actually tested.

Review the following before approving publication:

- Detection tests cover changed markers, precedence, unknown inputs, and probe failures. Matching
  identifiers name the actual input, and command execution remains opt-in.
- Public docs explain defaults, partial results, encoding, and emulation uncertainty. CLI tests
  cover argument handling, JSON contracts, and output failures.
- Dependency policy checks pass, and any exceptions have a specific rationale. Dependency updates
  preserve the declared MSRV and intended downstream behavior.
- Release notes describe the changes users receive, and the checkout has no unrelated edits.

Inspect both archives in `target/package/`. Each must contain its README, license texts, and source.
Check the normalized `Cargo.toml` files for the release version, MSRV, license, and repository URL.
The CLI archive must depend on the registry version of `detect-terminal`, without a checkout-only
path. Workspace packaging verifies the CLI against the library archive through Cargo's temporary
registry, including before the library's first publication.

Record the revision and check results in the release review. If source, metadata, or dependencies
change after validation, rerun the affected checks and require CI on the new revision.

## Publication and Verification

The `Release` workflow runs the full CI suite on `main`, then runs `release-plz release`. With
`release_always = false`, publication requires a merged release PR. Cargo publishes the library
before the CLI; release-plz creates the tag and GitHub release. The next job prepares or updates any
pending release PR. Ordinary development commits do not publish crates.

Publishing uses short-lived crates.io credentials obtained through GitHub OIDC. Both crates trust
`joshka/detect-terminal`, workflow `release-plz.yml`, and environment `crates-io`. GitHub restricts
that environment to the `main` branch. Only the publishing job has `id-token: write`; no Cargo token
or personal GitHub token is stored in repository secrets. The repository allows Actions to create
pull requests, and the PR job has the additional permission needed to dispatch CI.

After a successful release:

1. Confirm both versions are visible on crates.io and that docs.rs builds the library successfully.
1. Install the CLI into a fresh location with
   `cargo install detect-terminal-cli --version <version> --locked --root <directory>`.
1. Run the installed executable with `--help` and `--json`. Confirm the output matches the
   documented contract; terminal identity will depend on the environment where it runs.
1. Check that the `v<version>` tag points to the release revision and that the GitHub release
   contains the intended notes.

## Recover a Failed Release

Read the failed job logs and check which package versions reached crates.io before retrying.
Published versions cannot be overwritten; fixes to an uploaded package require a new version. For a
transient failure, rerun the failed workflow jobs on the same revision. Release-plz checks registry
state and skips versions already published. Verify both crates and the shared tag after any partial
release.

If authentication fails, compare the workflow filename and environment with each crate's trusted
publisher configuration, and check the environment's branch restriction. Renaming any of these
requires updating the corresponding configuration. Refer to the
[trusted publishing setup](https://release-plz.dev/docs/github/quickstart)
when changing the release workflow.
