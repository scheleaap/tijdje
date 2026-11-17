# Git tags are the version source of truth

Every push to `main` that contains a `feat`, `fix` or `perf` commit, or a breaking change, is released automatically. The next version is computed from the conventional commits since the last `v*` tag. CI never commits back to `main`: the release tag is the only record of the version. `Cargo.toml` therefore stays at `0.0.0` in git, and CI writes the real version into it just before the release build. Release notes live only on the GitHub release, not in a committed `CHANGELOG.md`.

## Considered Options

- Release PR (release-plz, release-please): a bot keeps a release PR with the version bump and `CHANGELOG.md` open. Gives control over when to release, but releasing on every push was preferred.
- Committing the version bump and changelog back to `main` from CI: keeps `Cargo.toml` accurate in git, but CI pushing to `main` loops on itself and clashes with branch rules.

## Consequences

- `cargo run` locally reports version `0.0.0`. Only release builds carry the real version.
- Commits that are not releasable on their own (`docs`, `chore`, …) appear in the notes of the next release.
- Dependabot merges use `GITHUB_TOKEN` and so don't trigger a run on `main`. Their `fix(deps)` commits ship with the next push.
