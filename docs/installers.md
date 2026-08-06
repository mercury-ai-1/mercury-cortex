# mercury-cortex installers

This directory contains the two official installers for the `mercury-cortex` CLI:

| File          | Platform            | Shell / Runtime                       |
|---------------|---------------------|---------------------------------------|
| `install.sh`  | Linux, macOS        | POSIX `sh` (dash, bash, zsh, …)        |
| `install.ps1` | Windows             | Windows PowerShell 5.1 + PowerShell 7  |

Both download prebuilt archives from **GitHub Releases**. They never build from
source and never execute downloaded content. Release artifacts and the
`checksums.txt` manifest must be present for the target version.

---

## 1. Installer ↔ release contract

A release `TAG` (for example `v0.5.2`) must carry these assets:

```
mercury-cortex-v0.5.2-x86_64-unknown-linux-gnu.tar.gz
mercury-cortex-v0.5.2-aarch64-unknown-linux-gnu.tar.gz
mercury-cortex-v0.5.2-x86_64-apple-darwin.tar.gz
mercury-cortex-v0.5.2-aarch64-apple-darwin.tar.gz
mercury-cortex-v0.5.2-x86_64-pc-windows-msvc.zip
checksums.txt
```

The binary inside a `.tar.gz` / `.zip` is named `mercury-cortex` (or
`mercury-cortex.exe` on Windows) and can sit at the archive root or under a
single top-level directory.

---

## 2. Release-URL construction

Every URL is derived from four constants: the repo slug, the release version,
the target triple, and the fixed asset prefix. There is **no endpoint scraping**
of release HTML — versions and asset names are assembled deterministically.

Given `REPO=owner/repo`, `VERSION=v0.5.2`, triple `x86_64-unknown-linux-gnu`:

```text
binary: https://github.com/<REPO>/releases/download/v0.5.2/mercury-cortex-v0.5.2-x86_64-unknown-linux-gnu.tar.gz
sums  : https://github.com/<REPO>/releases/download/v0.5.2/checksums.txt
```

- `install.sh` builds:
  - `ARCHIVE_NAME="${PROGRAM}-${VERSION_VALUE}-${TRIPLE}.tar.gz"`
  - `BASE_URL="https://github.com/${REPO}/releases/download/${VERSION_VALUE}"`
- `install.ps1` builds the same shape with `.zip` and the Windows triple.

The deterministic assembly keeps versions pinable in CI flows; the scripts run
headless by passing `VERSION=` / `-Version`.

## 3. Architecture-detection logic

Both scripts translate the host CPU into the **Rust target arch fragment** that
mirrors every artifact name.

Unix (`scripts/install.sh`):

- `uname -s` → OS fragment:
  - `Linux` → `unknown-linux-gnu`
  - `Darwin` → `apple-darwin`
  - anything else → **hard error** (no silent fallback)
- `uname -m` → arch fragment:
  - `x86_64` | `amd64` → `x86_64`
  - `aarch64` | `arm64` → `aarch64`
  - anything else → **hard error**

The final target triple is `ARCH-OS` (e.g. `aarch64-apple-darwin`), which maps
one-to-one onto the release asset name. `arm64` is explicitly accepted on macOS
because `uname -m` reports `arm64` there.

Windows (`scripts/install.ps1`):

- `$env:PROCESSOR_ARCHITECTURE`:
  - `AMD64` → `x86_64`
  - `ARM64` → `aarch64`
  - anything else (e.g. `x86`, `IA64`) → **hard error**

The triple is `ARCH-pc-windows-msvc`.

**Why arch matters:** Rust compiles platform-specific binaries; installing the
wrong arch fails with a confusing `Exec format error` / `Bad CPU type` at run
time rather than install time. Failing at install time with a clear message is
the better failure mode.

## 4. Checksum verification

Integrity is enforced **before** extraction:

1. Both installers fetch `checksums.txt` from the **same release tag** as the
   binary — never a different tag or endpoint.
2. They extract only the line referencing the exact archive name they
   downloaded, avoiding any trust in unrelated/forged lines.
3. They compute the SHA-256 of the downloaded archive locally:
   - shell: `sha256sum` (GNU coreutils) or `shasum -a 256` (macOS/BSD).
   - PowerShell: `Get-FileHash -Algorithm SHA256`.
4. Expected and actual digests are compared as lowercase strings.
5. Any mismatch → **immediate failure** (nonzero exit); the archive is never
   extracted or executed. Any missing entry in `checksums.txt` → failure.

`checksums.txt` is served over HTTPS, so checksum pinning primarily protects
against **corrupted/partial downloads** and adds a second line of defense
against tampered payloads beyond TLS. Signatures/attestations would be a
stronger layer (see section 7).

## 5. Security & operational hardening

- **Never execute downloaded content.** Extract → locate the plain binary →
  copy it verbatim (`install.sh`) / `Copy-Item` (PowerShell). Nothing is pipe
  to a shell interpreter.
- **Secure temporary area:**
  - shell: `mktemp -d` under `${TMPDIR:-/tmp}` with removal on an `EXIT` trap.
  - PowerShell: a per-process unique directory under `$env:TEMP`, removed in a
    `finally` block.
- **Fail fast:** `set -eu` in `install.sh`; `$ErrorActionPreference='Stop'` plus
  explicit `throw`/`exit 1` on every failure in `install.ps1`.
- **Atomic replace (shell):** the binary is copied to a `.new` staging name in
  the destination directory, then `mv -f` — a crash never leaves a half-written
  `mercury-cortex`. PowerShell uses `Copy-Item -Force` (overwrite) per platform.
- **Clean up even on error**, so no stray `.zip`, `.tar.gz`, or partial binary
  litters the machine.
- **No root required by default:** `/usr/local/bin` when writable, else
  `~/.local/bin` (Unix), or `%LOCALAPPDATA%\Programs\mercury-cortex-bin`
  (Windows, purely per-user).

## 6. Failure modes & messages

All errors go to `stderr` and the process exits nonzero. Common cases both
scripts handle explicitly:

| Condition                              | Behavior                                        |
|----------------------------------------|-------------------------------------------------|
| Unsupported OS / arch                  | Helpful message, exit 1.                        |
| Network / 404 on binary or checksum    | Message with the full URL, exit 1.             |
| `checksums.txt` missing our entry      | Message naming the asset, exit 1.              |
| Digest mismatch                        | Expected vs actual printed, exit 1.            |
| Destination not writable               | Tells user to use `sudo` / adjust perms.       |
| Extracted binary missing               | Message noting the expected name.              |

On success both print the installed path and the resolved version
(`mercury-cortex version` / `--version`).

## 7. Long-term maintenance improvements (no heavy complexity)

1. **Add a release workflow that emits and signs `checksums.txt`.**
   A GitHub Actions job building the five archives plus `checksums.txt`, then
   attaching **Sigstore/GitHub attestations** so `gh attestation verify` can
   anchor trust — converting TLS+checksum into supply-chain-attested
   verification.
2. **Keep `VERSION` / `-Version` pinning as the primary CI path.** Every CI
   install should pin the exact tag (not "latest") for reproducibility;
   "latest" remains the interactive convenience.
3. **Add a self-update subcommand** (e.g. `mercury-cortex upgrade`) so users can
   re-run install logic without the curl one-liner.
4. **Run `shellcheck` and `PSScriptAnalyzer` in CI** against both scripts to
   catch regressions exactly as `install.sh` was validated here.
5. **Document the curl | sh trade-off** and pin a documented script content
   hash per release for scriptable installs.

Keep the scripts dependency-free (curl + tar / built-in PowerShell cmdlets) —
that is why they run anywhere with zero setup cost.