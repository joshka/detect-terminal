# Releasing

A release candidate must meet the criteria below. Local checks establish local readiness; hosted
platform results and registry access are separate gates before publishing.

## Acceptance Criteria

| Criterion                            | Required evidence                                                                                                        |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| Detection is predictable             | Tests cover recognized hints, precedence, empty/unknown values, and near-miss `TERM` names                               |
| Evidence is truthful                 | Matching identifiers and sources name the actual environment key or successful command                                   |
| Defaults have bounded work           | Default detection reads only the environment; blocking commands require explicit opt-in                                  |
| Probe failures degrade clearly       | Missing tools, nonzero exits, and malformed output leave metadata absent and retain diagnostics                          |
| Public contracts are clear           | Defaults, encoding, partial results, capture limits, and emulation uncertainty are documented                            |
| CLI is usable in scripts             | Argument, JSON, human output, and broken-pipe tests pass; JSON schema version is documented                              |
| Workspace checks pass                | `just check` passes with warning-free public/private Rustdoc and strict Clippy                                           |
| Supported compiler works             | `just msrv` passes on the declared MSRV; CI also checks it with Clippy                                                   |
| Supported hosts work                 | Native Linux, macOS, and Windows CI tests pass for the exact release revision                                            |
| Dependency maintenance is controlled | Lockfile updates pass checks; advisories are reviewed; manifest requirement changes are deliberate                       |
| Archives are self-contained          | `just package` verifies both crates; extracted archives contain README, license texts, source, and correct metadata      |
| Published metadata is accurate       | License, repository, descriptions, versions, MSRV, and the CLI's library requirement match the release                   |
| Release is reviewable                | Every `jj` change has a purpose, there are no conflicts or unintended files, and release notes describe shipped behavior |
| Publication is possible              | Repository exists, crate names and owner access are confirmed, and publish dry-run succeeds                              |

The current MSRV is Rust 1.97. The policy allows moving to the previous stable release when a real
implementation or dependency requirement arises. It does not require a bump for every release.

## Local Validation

```sh
just check
just msrv
just ci-check
just audit
just package
cargo publish --workspace --dry-run --locked
```

Packaging and workspace dry-run use current stable Cargo. They test the CLI against the library
archive through Cargo's temporary registry, including before the first library version exists on
crates.io. On a working change, `cargo package --workspace --allow-dirty --locked` can validate the
current files; review the actual revision before publishing.

Inspect `target/package/*.crate`, not only the source checkout. Each crate links to the root license
texts so Cargo includes them in the archive. The library README is included from the workspace root;
the CLI has its own installation and output contract. Confirm the normalized manifests refer to the
registry version of `detect-terminal` rather than a checkout-only path.

## Publication

1. Review the `jj` stack and update the Unreleased notes with the version and release date.
1. Confirm the GitHub repository and remote, publish the reviewed revision, and wait for all CI
   jobs.
1. Recheck crate-name availability and publishing permissions. A name being available earlier is not
   a reservation.
1. Run the local gates and publish dry-run against that revision.
1. Publish the workspace with current stable Cargo. It orders the library before the CLI dependency.
1. Verify the published library docs and install the CLI from crates.io in a fresh location.
1. Tag the released revision and publish release notes tied to that revision.

Publishing is a separate maintainer action. Neither the justfile nor CI uploads crates or creates a
release automatically.

## Current Candidate Evidence

As of 2026-09-26, local macOS validation passed on stable Rust and Rust 1.97: 73 library tests, one
CLI unit test, six CLI integration tests, and 11 doctests. Strict Clippy, public and private
Rustdoc, formatting checks, workflow syntax validation, the dependency advisory audit, archive
verification, and the workspace publish dry-run also passed. The archives contain the license texts
and registry dependency metadata.

Native Linux and Windows CI results are still required. The intended repository is
`joshka/detect-terminal`; it did not exist at the readiness check. Both crate names returned
not-found from crates.io at that time. These observations do not establish future availability or
publishing permissions. No crates have been published by this cleanup.
