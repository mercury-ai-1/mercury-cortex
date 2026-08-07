# Mercury Cortex Release Pipeline

This document is the operational guide for the CI/CD pipeline. It corresponds
to the approved design in
[`docs/superpowers/specs/2026-08-06-release-pipeline-design.md`](./superpowers/specs/2026-08-06-release-pipeline-design.md).

---

## 1. Overall workflow architecture

Four workflows, one responsibility each. They never publish from PRs, and no
workflow contacts crates.io except `release.yml`.

```
.github/workflows/
├── ci.yml               # PR + push to main: fmt, clippy, test, audit
├── build.yml            # version tags (v*): build 5 targets → archive → attest → upload
├── release.yml          # workflow_call (from build.yml): GitHub Release + crates.io
└── installer-test.yml   # workflow_run (on build.yml): verify installers on released binaries
```

### Dependency graph

```
PR / push main ─────────► ci.yml            fmt · clippy · test · audit
                                       (no publish)

git tag v0.1.0 ──────────► build.yml       linux-x64 · linux-arm64(cross) ·
        │                                    macos-x64 · macos-arm64 · windows-x64
        │                                    └ upload-artifact → attest-build-provenance
        ▼
        release.yml  (final job, runs INSIDE build.yml)
        │  → GitHub Release (archives + checksums.txt)
        │  → publish mercury-cortex              (only after release)
        ▼
   build.yml run completes
        ▼
installer-test.yml      linux install.sh · macos install.sh · windows install.ps1
```

**Key invariant:** because `release.yml` runs as the *final job of* `build.yml`
and `installer-test.yml` watches `build.yml` via `workflow_run`, the installers
are **only ever tested against a release that already exists on GitHub**.

---

## 2. Recommended GitHub Actions

All actions are pinned to **full commit SHAs** for supply-chain safety. Each
`# <owner>/<repo>@<tag>` comment records the human-readable version;
Dependabot keeps the SHAs current.

| Purpose                     | Action (commit SHA)                                  |
|-----------------------------|------------------------------------------------------|
| checkout sources            | `actions/checkout@v4`                                |
| install Rust toolchain      | `dtolnay/rust-toolchain@stable`                      |
| Rust build cache            | `Swatinem/rust-cache@v2`                            |
| cross-compile aarch64 Linux | `taiki-e/install-action@v2` (`cross`)               |
| audit / auditable tooling   | `taiki-e/install-action@v2` (`cargo-audit`, `cargo-auditable`) |
| upload build artifacts      | `actions/upload-artifact@v4`                        |
| download build artifacts    | `actions/download-artifact@v4`                      |
| SLSA attestation            | `actions/attest-build-provenance@v2`                 |

The current pinned SHAs are recorded inline in each workflow file.

---

## 3. Build strategy

| Target                      | Runner          | Method          |
|-----------------------------|-----------------|-----------------|
| `x86_64-unknown-linux-gnu`  | `ubuntu-latest` | native          |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross` (QEMU)  |
| `x86_64-apple-darwin`       | `macos-13`      | native          |
| `aarch64-apple-darwin`      | `macos-14`      | native (arm64)  |
| `x86_64-pc-windows-msvc`    | `windows-latest`| native          |

`cross` is used **only** for the Linux ARM64 leg, the single case where a
native GitHub-hosted runner isn't available. Everything else builds natively
(fastest, most reliable). `cargo-zigbuild` was considered but rejected:
`cross` is the ecosystem standard and adds no toolchain step to 4 of 5 legs.

### Packaging & checksums

- Each leg packages a single archive: Linux/macOS → `.tar.gz`, Windows → `.zip`.
  Archive/triple match exactly what `scripts/install.sh` and
  `scripts/install.ps1` expect (see `docs/installers.md`).
- **`checksums.txt` is generated centrally in `release.yml`** on one Linux job
  after all five archives are downloaded; `sha256sum` output is then
  deterministic and cross-platform pain is avoided.
- **Attestations** (`actions/attest-build-provenance`) attach SLSA provenance
  to every archive in `build.yml`. These and the SHA-256 checksums together let
  users verify `gh attestation verify` and the installers' checksum check.

---

## 4. Triggers / versioning strategy

- **Releases are Git tags only** (`v0.1.0`, `v0.2.0`, `v1.0.0`). The tag name
  *is* the version; there is no other source of truth.
- Release flow runs only on `push: tags: ['v*']`. PRs push to `main` and
  `/workflows patterns` never trigger it.
- `release.yml` cannot be triggered by any `on:`; it is called exclusively by
  the `release` job of `build.yml` (`uses: ./.github/workflows/release.yml`).
- `installer-test.yml` triggers on `workflow_run` of `build.yml`, gated to
  `conclusion == success` (plus a `conclusion` guard per job). `workflow_run`
  always runs against the default branch.

> Note: `workflow_run.head_branch` carries the tag name for tag-triggered runs,
> which `installer-test.yml` passes as `VERSION` to pin the exact build being
> tested.

---

## 5. Security posture

- **Minimal `permissions:`** on every workflow. `contents: write` exists only
  in `release.yml`; `id-token`/`attestations` write only in `build.yml`.
- **Actions pinned to SHAs** (Dependabot maintains them).
- **`concurrency`** groups: CI cancels superseded runs; release + installer
  runs never cancel (so a long publish is not killed by a retry).
- **`CARGO_REGISTRY_TOKEN`** is passed into `release.yml` as an input secret
  (`secrets: inherit`) and is used by no other workflow.
- **Artifact integrity:** SHA-256 checksums + SLSA attestation on every
  release binary; installers verify checksums before extraction.
- **Fail-safe order:** GitHub Release (assets) is created before publishing;
  publishing runs only `needs`-after release creation.
- **Duplicate-release prevention:** `gh release view` skips an already-created
  release; crates.io `cargo search` gates re-publishing an existing version.
  Re-running a failed tag is therefore safe and idempotent.

---

## 6. Required repository settings

1. **Branch protection on `main`:** require the `fmt`, `clippy`, `test`,
   `audit` checks from `ci.yml`.
2. **Workflow permissions:** Repository → Settings → Actions → "Read
   repository contents and packages" (explicit least-privilege, not the
   default read/write).
3. **Tag protection:** prevent force-pushing to or deleting `v*` tags (GitHub
   rule) so released tags stay immutable.
4. **Environments:** create a **`release`** environment (Settings →
   Environments). `release.yml` jobs declare `environment: release`, which
   allows adding a *required reviewer* gate (manual approval) later without
   changing the workflows.
5. **Dependabot:** enable version updates for GitHub Actions (and cargo).

---

## 7. Required Cargo.toml changes

Already applied. Summary:

- Uses the crates.io dependency (no git dependency):
  ```toml
  mercury-cortex-core = "0.1.0"
  ```
- Added crates.io-required metadata: `description`, `license = "Apache-2.0"`,
  `repository`, `readme = "README.md"`, `keywords`, `categories`.
- Added `exclude` for `.github/`, `docs/`, `graphify-out/`, `tests/`,
  `scripts/`, keeping the crate tarball clean.
- Added a release profile:
  ```toml
  [profile.release]
  lto = "thin"
  codegen-units = 1
  strip = "symbols"
  ```

> `mercury-cortex-core` is published by its own repository and workflow on
> crates.io; `release.yml` does not publish or manage it.

---

## 8. Future expansion (intentionally not implemented)

- **Homebrew tap:** a `brewpods` formula templating the release URL +
  checksum; identical archive naming already supports it.
- **WinGet manifest:** a manifest pointing at the Windows zip + checksum; the
  install dir should match `install.ps1`.
- **Docker:** a multi-arch `ghcr.io/mercury-ai-1/mercury-cortex` image; the
  Linux GNU archive already matches a scratch/glibc base.
- **Scoop manifest:** a JSON manifest referencing the Windows zip.
- **AUR package:** PKGBUILD pulling the Linux release archive.
- **Gate Windows legs once `src/svc` is portable** (see design doc §Windows):
  flip the `continue-on-error` flags in `build.yml` / `installer-test.yml` and
  add Windows to the `ci.yml` test matrix.
- **Adopt a release-signing/releasing TF-M or verifier -count** after the first
  stable release if the project grows.

Whatever you add later, keep the invariant: **one release created per tag,
one source of truth (the tag), and installers that only consume released
assets.**