# Releasing

The workspace publishes two crates at a shared version: the `detect-terminal` library and the
`detect-terminal-cli` package, which installs the `detect-terminal` executable. Releases are manual;
CI validates changes but does not upload crates or create releases.

Use current stable Cargo for packaging and publishing. The minimum supported compiler is a separate
compatibility requirement; see [Contributing](../CONTRIBUTING.md#setup) for the MSRV policy and tool
setup.

## Prepare the Release

1. Review changes since the previous release for API compatibility, detection precedence, command
   behavior, CLI arguments, and JSON output. Choose a version that reflects their impact on callers.
1. Set `workspace.package.version` in the root `Cargo.toml` and update the CLI's `detect-terminal`
   dependency requirement in `crates/detect-terminal-cli/Cargo.toml` to the release version. Both
   packages inherit the workspace version. Refresh their lockfile entries with
   `cargo check --workspace` and inspect the diff for unrelated dependency changes.
1. Turn the Unreleased section in `CHANGELOG.md` into release notes with the version and date.
   Explain behavior changes and any migration steps. Align the README, CLI help, and Rustdoc with
   what ships.
1. Confirm the repository URL and package metadata are accurate. Before the first release, create
   the public repository and confirm that both crate names are available. For later releases,
   confirm publishing access to both existing crates.
1. Commit the release preparation and push the revision for CI. Publish from a clean checkout of
   that exact revision.

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

## Publish and Verify

With publishing credentials configured for crates.io, run:

```sh
cargo publish --workspace --locked
```

Cargo publishes the library before the CLI that depends on it. If publication stops partway through,
check which versions reached crates.io before retrying. Published versions cannot be overwritten;
fixes to an uploaded package require a new version.

1. Confirm both versions are visible on crates.io and that docs.rs builds the library successfully.
1. Install the CLI from crates.io into a fresh location, selecting the released version explicitly
   with `cargo install detect-terminal-cli --version <version> --locked --root <directory>`.
1. Run the installed executable with `--help` and `--json`. Confirm the output matches the
   documented contract; terminal identity will depend on the environment where it runs.
1. Tag the validated revision as `v<version>` and create a GitHub release using the changelog notes.
1. Restore an empty Unreleased section for subsequent changes.
