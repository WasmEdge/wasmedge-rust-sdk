# Contributing guidelines

## How to become a contributor and submit your own code

### Developer Certificate of Origin (DCO)

We'd love to accept your patches! Before we can take them, you are often asked to sign a DCO (Developer Certificate of Origin) to ensure that the project has the proper rights to use your code. [A Complete Guide to DCO for Open Source Developers](https://www.secondstate.io/articles/dco/) tells you how to do it. Please read it carefully before you start your work.

### GitHub Issues

If you want to work on a GitHub issue, check to make sure it's not assigned to someone first.
If it's not assigned to anyone, assign yourself once you start writing code.
(Please don't assign yourself just because you'd like to work on the issue, but only when you actually start.)
This helps avoid duplicate work.

If you start working on an issue but find that you won't be able to finish, please un-assign yourself so other people know the issue is available.
If you assign yourself but aren't making progress, we may assign the issue to someone else.

If you're working on issue 123, please put "Fixes #123" (without quotes) in the commit message below everything else and separated by a blank line.
For example, if issue 123 is a feature request to add foobar, the commit message might look like:

```text
Add foobar

Some longer description goes here, if you
want to describe your change in detail.

Fixes #123
```

This will [close the bug once your pull request is merged](https://help.github.com/articles/closing-issues-using-keywords/).

If you're a first-time contributor, try looking for an issue with the label "good first issue", which should be easier for someone unfamiliar with the codebase to work on.

### Git

Please check out a recent version of `main` before starting work, and rebase onto `main` before creating a pull request.
This helps keep the commit graph clean and easy to follow. In addition, please sign off each of your commits.

## Building and testing

This workspace has five publishable crates: `wasmedge-sdk` (repo root), and
`wasmedge-sys`, `wasmedge-types`, `wasmedge-macro`, `async-wasi` under `crates/`.
`wasmedge-sdk` and `wasmedge-sys` link against the WasmEdge C library; the
`bundled` feature downloads and statically links a matching build automatically,
so it's the easiest way to get a working local build without installing anything
system-wide.

Requirements: Rust **1.85** or newer (stable channel — this is the workspace
MSRV, enforced by `rust-version` in every crate's manifest), and on Linux the
`libzstd`/`libfmt` headers (see the README's "Build requirements" for
distro-specific package names).

### Quick verification

This mirrors what `guardrails.yml` and the `ci-build*` workflows check on every
PR; run it before opening one:

```bash
cargo build --release --features bundled
cargo test --release --features bundled --workspace --exclude async-wasi -- --test-threads=1 --skip test_vmbuilder
cargo clippy --workspace --exclude async-wasi --lib --examples --features bundled,aot,ffi,wasi_nn,wasmedge_process -- -D warnings
cargo fmt --all -- --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --features standalone,aot,ffi
```

`async-wasi` itself is excluded from the workspace-wide test/clippy runs above
because its interesting behavior (sockets, signals, timeouts) is Linux-specific;
if you're on macOS or Windows, cross-clippy it against Linux instead so you don't
fly blind on `cfg(unix)` code:

```bash
cargo clippy -p async-wasi --target x86_64-unknown-linux-gnu --all-targets -- -D warnings
```

Either way, push and let the `ubuntu-latest` integration CI run the real
Linux-native test suite — it's the only environment that exercises every
platform-gated code path.

### Checking public API stability

This project targets zero accidental breaking changes on 0.x releases. Before
sending a PR that touches a publishable crate's public surface, run
[`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks)
against the last **published** version of that crate:

```bash
cargo semver-checks check-release -p <crate> --baseline-version <last-published-version>
# wasmedge-sys and wasmedge-sdk link against libwasmedge, so also pass:
cargo semver-checks check-release -p wasmedge-sys --baseline-version <last-published-version> --features standalone
cargo semver-checks check-release -p wasmedge-sdk --baseline-version <last-published-version> --features standalone
```

`wasmedge-macro` is a proc-macro crate, which `cargo-semver-checks` cannot
analyze; it needs `trybuild`/`macrotest`-style UI tests as its guardrail
instead. That test infrastructure does not exist yet — until it does, review
changes to `wasmedge-macro`'s expansion output by hand.

Findings are informational, not automatically blocking — some documented,
intentional exceptions exist (recorded in `docs/Upgrade_to_0.17.0.md` and the
CHANGELOG). Anything else `cargo-semver-checks` flags needs either a fix or an
explicit, reviewed justification before merging.

## Releasing

Releases are manual, one crate at a time, via each crate's `Release <crate>
crate` GitHub Actions workflow (`.github/workflows/release-*.yml`). Every one of
them takes a `dry_run` input (defaults to `true`, meaning `cargo publish
--dry-run`); `wasmedge-sys` and `wasmedge-sdk` additionally take a
`wasmedge_version` input pinning which WasmEdge C API release to validate
against.

**Release order matters** because of the dependency graph — release leaves
before the things that depend on them:

1. `wasmedge-types` (no workspace-internal dependencies)
2. `wasmedge-macro` (no workspace-internal dependencies)
3. `async-wasi` (no workspace-internal dependencies; only linux-gated consumers
   depend on it)
4. `wasmedge-sys` (depends on `wasmedge-types`; optionally `async-wasi`)
5. `wasmedge-sdk` (depends on all four of the above)

For each crate, in order:

1. Bump its `version` in `Cargo.toml`, and bump the version constraint on it in
   every workspace member that depends on it via a `path` + `version` dependency
   (e.g. bumping `wasmedge-types` also means bumping the `wasmedge-types`
   version requirement in `wasmedge-sys`'s and `wasmedge-sdk`'s manifests).
2. Move the relevant entries out of the CHANGELOG's `## [Unreleased]` section
   into a new `## [<version>] - <date>` section.
3. Dispatch the crate's release workflow with `dry_run: true`. Confirm it's
   green — this runs `cargo publish --dry-run` (and, for `wasmedge-sys`/
   `wasmedge-sdk`, a real build/test/doc pass against the pinned WasmEdge
   version) without publishing anything.
4. Dispatch it again with `dry_run: false` to actually publish to crates.io.
5. Tag the release commit and push the tag: `vX.Y.Z` for `wasmedge-sdk` (the
   project's "main" tag), `<crate>-vX.Y.Z` for every other crate — e.g.
   `wasmedge-sys-v0.21.0`, `wasmedge-types-v0.7.0`, `wasmedge-macro-v0.7.0`,
   `async-wasi-v0.3.0`. This convention is what `cliff.toml`'s `tag_pattern` is
   written to match; the repository's older tags (bare `X.Y.Z`, `sys/X.Y.Z`,
   `async-wasi/X.Y.Z`, ...) predate it and are not being renamed retroactively.
6. Move on to the next crate in the order above.

If a mid-sequence step fails, later crates in the order haven't been touched
yet, and earlier ones are already published — recover with a patch release of
the failed crate rather than trying to unpublish anything (crates.io publishes
are permanent).
