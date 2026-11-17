# CI/CD pipeline

Status: ready-for-agent

## Problem Statement

tijdje is published on GitHub, but nothing checks it automatically and there are no releases. Tests, formatting and lints run only when I remember to run them locally. Getting a binary means building from source. Versions and changelogs don't exist. The Rust version is set to `latest` in the mise config, so builds aren't reproducible. Dependencies go stale unless I update them by hand.

## Solution

A GitHub Actions pipeline that:

- On every PR and every push to `main`, checks formatting, lints and tests with the Rust version pinned in the mise config.
- On a push to `main` that contains releasable commits, computes the next semantic version from the conventional commit messages. It then builds a Linux binary and publishes a GitHub release with the binary, its checksum and release notes listing every change since the previous release.
- Has Dependabot propose monthly updates for actions and Cargo dependencies, which merge automatically once CI passes.

Every action is pinned to an immutable commit SHA. The repo allows only squash merges, and `main` has a ruleset that makes PRs wait for CI.

## User Stories

1. As the maintainer, I want CI to run on every PR, so that I see breakage before merging.
2. As the maintainer, I want CI to run on every push to `main`, so that direct pushes are checked too.
3. As the maintainer, I want CI to fail when code isn't formatted with rustfmt, so that formatting stays consistent.
4. As the maintainer, I want CI to fail on any clippy warning, so that lint issues don't pile up.
5. As the maintainer, I want CI to run all tests, so that regressions are caught.
6. As the maintainer, I want CI to build with the locked dependency versions, so that CI tests what I test locally.
7. As the maintainer, I want CI to use the Rust version from the mise config, so that I maintain the Rust version in one place.
8. As the maintainer, I want the mise config to pin a specific Rust version instead of `latest`, so that builds are reproducible.
9. As the maintainer, I want CI to install only Rust from the mise config, not Node or other tools, so that CI stays fast and lean.
10. As the maintainer, I want every action pinned to a full commit SHA with the version in a comment, so that a moved tag can't change what runs.
11. As the maintainer, I want a push to `main` with a `feat`, `fix` or `perf` commit, or a breaking change, to produce a release automatically, so that I never cut releases by hand.
12. As the maintainer, I want a push with only non-releasable commits (`docs`, `chore`, `test`, `ci`, …) to run CI without releasing, so that releases mean something.
13. As the maintainer, I want the next version computed from conventional commits since the last release tag, so that versioning follows semver without thought.
14. As the maintainer, I want `feat` to bump the minor version and `fix`/`perf` the patch version, so that versions follow the 0.x convention.
15. As the maintainer, I want breaking changes to bump only the minor version while on 0.x, so that I don't jump to 1.0 by accident.
16. As the maintainer, I want the first release to be `v0.1.0`, so that the history starts at a sensible version.
17. As the maintainer, I want releases tagged `vX.Y.Z`, so that tags follow the usual convention.
18. As the maintainer, I want a release only after the checks pass, so that broken code is never released.
19. As the maintainer, I want releases from back-to-back pushes to run one at a time, so that two runs never compute the same version.
20. As the maintainer, I want the release notes to list every commit since the previous release, so that nothing is hidden.
21. As the maintainer, I want the release notes grouped by commit type (Features, Bug fixes, Performance, Refactoring, Documentation, Tests, Build and CI, …), so that they're easy to scan.
22. As the maintainer, I want non-releasable commits to appear in the next release's notes, so that they're still recorded.
23. As the maintainer, I want breaking changes marked in the release notes, so that they stand out.
24. As the maintainer, I want the changelog only in GitHub release notes, not in a committed file, so that CI never pushes to `main`.
25. As the maintainer, I want the crate version in git to stay at `0.0.0`, so that nobody mistakes it for the real version (see ADR-0002).
26. As the maintainer, I want CI to write the real version into the crate just before the release build, so that the released binary carries it.
27. As a user, I want to download a ready-built Linux x86_64 binary from the release page, so that I don't need a Rust toolchain.
28. As a user, I want the binary named `tijdje-vX.Y.Z-x86_64-unknown-linux-gnu`, so that I can tell the version and platform.
29. As a user, I want a `.sha256` file next to the binary, so that I can verify my download with `sha256sum -c`.
30. As a user, I want the source archives GitHub attaches to every release, so that the GPL source is available with each binary.
31. As the maintainer, I want Dependabot to propose action updates monthly, so that pinned SHAs don't go stale.
32. As the maintainer, I want Dependabot to propose Cargo dependency updates monthly, so that dependencies stay current.
33. As the maintainer, I want action updates committed as `chore(deps)`, so that they don't trigger a release on their own.
34. As the maintainer, I want Cargo updates committed as `fix(deps)`, so that they ship with the next release.
35. As the maintainer, I want Dependabot PRs to merge automatically once CI passes, so that I don't babysit them.
36. As the maintainer, I want Dependabot PRs with failing CI to stay open, so that I can look into them.
37. As the maintainer, I want only squash merges allowed, so that `main` has one conventional commit per PR.
38. As the maintainer, I want auto-merge turned on in the repo, so that Dependabot PRs can merge themselves.
39. As the maintainer, I want a ruleset on `main` that requires the CI check before a PR merges, so that auto-merge waits for green CI.
40. As the maintainer, I want to keep pushing directly to `main`, so that the ruleset doesn't slow down my own workflow.
41. As the maintainer, I want to see the exact `gh` commands for the repo settings before they're applied, so that I stay in control of my repo.

## Implementation Decisions

- **Workflows:** a CI workflow with a `check` job (fmt, clippy with warnings as errors, tests, all with locked dependencies) and a `release` job that depends on `check` and runs only for pushes to `main`. A separate workflow handles Dependabot auto-merge.
- **Rust toolchain:** CI installs Rust via the mise action, limited to the `rust` tool. The mise config pins the current stable version (1.99.1 at time of writing) instead of `latest`. rustfmt and clippy components are ensured explicitly.
- **Action pinning:** every `uses:` refers to a full commit SHA, with the release tag in a trailing comment. Only the checkout and mise actions are needed. Releases are created with the preinstalled `gh` CLI, not a release action.
- **Versioning and notes:** git-cliff, a pinned version run through mise in CI (not added to the project's mise config). A git-cliff config at the repo root:
  - Conventional commits parsing, unconventional commits not filtered out.
  - Tag pattern `v[0-9].*`.
  - Commit groups in a fixed order, with a catch-all "Other" group.
  - Bump rules: `feat` → minor, breaking → minor (0.x), initial tag `v0.1.0`.
- **Release decision:** the release job checks the commits since the last `v*` tag (or since the start, when there is none) for a `feat`, `fix` or `perf` subject, a `!` breaking marker, or a `BREAKING CHANGE` footer. If none is found, it logs that and skips the remaining steps. The decision isn't delegated to git-cliff, because git-cliff may bump the patch version for any commit.
- **Release build:** the crate version is set to the computed version in the CI workspace only. Then a release build, and the binary is copied to the asset name with a SHA-256 checksum file. The release is created with `gh`, targeting the pushed commit SHA, titled with the version, with the git-cliff unreleased notes as body.
- **Concurrency:** the release job is in a single concurrency group that doesn't cancel runs in progress.
- **Permissions:** read-only contents by default. The release job gets contents write. The auto-merge workflow gets contents and pull-requests write.
- **Dependabot:** monthly updates for the `github-actions` and `cargo` ecosystems. Commit prefixes `chore` and `fix` respectively, with scope (yielding `chore(deps)` / `fix(deps)`).
- **Auto-merge:** on PRs opened by Dependabot, enable auto-merge with squash via `gh`.
- **Repo settings, applied with `gh` after showing the commands:**
  - Allow auto-merge.
  - Disable merge commits and rebase merges, leaving squash only.
  - A ruleset on `main` requiring the `check` status check, with an always-bypass for the repository admin role, so direct pushes keep working.
- **Known consequence:** auto-merged Dependabot commits are pushed with `GITHUB_TOKEN`, so they don't trigger a run on `main`. Their changes ship with the next push. No personal access token is introduced.
- Recorded in ADR-0002 (git tags are the version source of truth).

## Testing Decisions

- A good test observes the pipeline's external behavior (check results, releases, assets, merges), not the workflow's internals.
- **Seam 1, the workflows on GitHub**, verified by real runs:
  - A throwaway PR with a deliberate formatting error fails `check` and can't be merged.
  - The first push to `main` publishes `v0.1.0` with the binary, its checksum and grouped notes. The downloaded binary passes `sha256sum -c` and runs.
  - A `docs`-only push runs CI and skips the release.
  - A Dependabot PR auto-merges after `check` passes.
- **Seam 2, local git-cliff dry-run** against this repo before pushing: run the same bumped-version and unreleased-notes commands CI runs, and confirm `v0.1.0` and sensible grouped notes.
- No new Rust tests: no application code changes. The existing integration tests under the tests directory run as part of `check`.
- No prior art: this is the repo's first CI.

## Out of Scope

- Platforms other than Linux x86_64 (aarch64, macOS, Windows).
- Archives (`.tar.gz`) or bundling README/LICENSE with the binary.
- A committed `CHANGELOG.md`, release PRs, or CI commits to `main`.
- Publishing to crates.io or package managers.
- A `--version` flag in the application. The version is set in the build but not yet shown.
- Releasing Dependabot Cargo bumps immediately (would need a personal access token or GitHub App).
- Build caching beyond what the mise action provides.

## Further Notes

- `gh` is logged in. The repo is public, auto-merge is currently off, all three merge methods are allowed, and `main` has no protection.
- The mise config also lists Node and the `skills` npm tool at `latest`. Those stay untouched.
- Running git-cliff locally (seam 2) means installing it temporarily via mise. Ask first.
- Action versions at time of writing: checkout v7.0.1, mise-action v5.2.9, git-cliff 2.14.2. Re-resolve SHAs when implementing.
- The skills sync may overwrite the local addition to the grill-with-docs skill.
