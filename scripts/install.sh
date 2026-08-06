#!/bin/sh
# install.sh — Install `mercury-cortex` from GitHub Releases into /usr/local/bin
# or ~/.local/bin on Linux and macOS.
#
#   * Downloads the correct prebuilt archive for this OS/arch.
#   * Resolves the latest GitHub release (or a pinned VERSION).
#   * Verifies the SHA-256 checksum from `checksums.txt` before extraction.
#   * Never builds from source. Never executes downloaded content.
#   * Fully non-interactive; clean failures; temporary files always removed.
#
# Environment variables (all optional):
#   VERSION                      Pin a specific release tag, e.g. "v0.5.2".
#                                Defaults to the latest release.
#   GITHUB_REPO                  Owner/repo, e.g. "mercury-ai-1/mercury-cortex".
#   MERCURY_CORTEX_INSTALL_DIR   Override the destination directory entirely.
#
# Examples:
#   curl -fsSL https://raw.githubusercontent.com/mercury-ai-1/mercury-cortex/main/scripts/install.sh | sh
#   VERSION=v0.5.2 sh scripts/install.sh

set -eu

PROGRAM="mercury-cortex"
DEFAULT_REPO="mercury-ai-1/mercury-cortex"
REPO="${GITHUB_REPO:-${DEFAULT_REPO}}"
VERSION="${VERSION:-}"

# ---------------------------------------------------------------------------
# Logging helpers
# ---------------------------------------------------------------------------
say() { printf '%s\n' "$*"; }
err() { printf '%s: error: %s\n' "$PROGRAM" "$*" >&2; }
die() { err "$*"; exit 1; }

TMP_DIR=""
cleanup() {
  [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ] && rm -rf -- "$TMP_DIR"
}
trap cleanup EXIT HUP INT TERM

# ---------------------------------------------------------------------------
# Operating-system detection (Rust target OS fragment).
# ---------------------------------------------------------------------------
detect_os() {
  case "$(uname -s 2>/dev/null || true)" in
    Linux)  OS="unknown-linux-gnu" ;;
    Darwin) OS="apple-darwin" ;;
    *)
      err "unsupported operating system (supported: Linux, macOS)"
      exit 1
      ;;
  esac
}

# ---------------------------------------------------------------------------
# CPU-architecture detection (Rust target arch fragment).
# ---------------------------------------------------------------------------
detect_arch() {
  case "$(uname -m 2>/dev/null || true)" in
    x86_64|amd64)   ARCH="x86_64" ;;
    aarch64|arm64)  ARCH="aarch64" ;;
    *)
      err "unsupported CPU architecture (supported: x86_64, aarch64)"
      exit 1
      ;;
  esac
}

# ---------------------------------------------------------------------------
# Resolve the version to install:
#   * pinned VERSION, if given,
#   * otherwise the latest tag from the GitHub Releases API.
# ---------------------------------------------------------------------------
resolve_version() {
  if [ -n "$VERSION" ]; then
    case "$VERSION" in
      v*) VERSION_VALUE="$VERSION" ;;
      *)  VERSION_VALUE="v${VERSION}" ;;
    esac
    say "Installing pinned version $VERSION_VALUE"
    return
  fi

  command -v curl >/dev/null 2>&1 || die "curl is required to resolve the latest release"
  say "Resolving latest release of $REPO ..."
  VERSION_VALUE="$(
    curl --silent --fail --show-error \
      "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
      | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
      | head -n 1
  )"
  [ -n "$VERSION_VALUE" ] || {
    err "could not resolve the latest release tag for $REPO"
    err "pin one explicitly with VERSION=v0.1.0"
    exit 1
  }
  say "latest release: $VERSION_VALUE"
}

# ---------------------------------------------------------------------------
# Locate an SHA-256 tool: prefer sha256sum (Linux coreutils), fall back to
# shasum -a 256 (macOS).
# ---------------------------------------------------------------------------
SUM_PROG=""
detect_sum_tool() {
  if command -v sha256sum >/dev/null 2>&1; then
    SUM_PROG="sha256sum"
  elif command -v shasum >/dev/null 2>&1; then
    SUM_PROG="shasum -a 256"
  else
    err "no SHA-256 tool found (install 'sha256sum' or 'shasum')"
    exit 1
  fi
}

sha256_of() {
  # shellcheck disable=SC2086
  $SUM_PROG "$1" 2>/dev/null | awk '{ print $1 }'
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
main() {
  detect_os
  detect_arch
  resolve_version
  detect_sum_tool

  TRIPLE="${ARCH}-${OS}"
  ARCHIVE_NAME="${PROGRAM}-${VERSION_VALUE}-${TRIPLE}.tar.gz"
  SUM_FILENAME="checksums.txt"
  BASE_URL="https://github.com/${REPO}/releases/download/${VERSION_VALUE}"
  ARCHIVE_URL="${BASE_URL}/${ARCHIVE_NAME}"
  SUM_URL="${BASE_URL}/${SUM_FILENAME}"

  say ""
  say "Installing $PROGRAM"
  say "  version : $VERSION_VALUE"
  say "  target  : $TRIPLE"
  say "  source  : $ARCHIVE_URL"
  say ""

  command -v curl >/dev/null 2>&1 || die "curl is required to download the binary"
  command -v tar >/dev/null 2>&1 || die "tar is required to extract the binary"

  TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/${PROGRAM}.XXXXXX")" \
    || die "failed to create a temporary directory"
  cd "$TMP_DIR"

  # --- Download ---------------------------------------------------------------
  say "Downloading archive..."
  curl --fail --show-error --location --silent --output "$ARCHIVE_NAME" "$ARCHIVE_URL" \
    || die "failed to download $ARCHIVE_URL"

  say "Downloading checksums..."
  curl --fail --show-error --location --silent --output "$SUM_FILENAME" "$SUM_URL" \
    || die "failed to download $SUM_URL"

  # --- Verify checksum --------------------------------------------------------
  say "Verifying SHA-256 checksum..."
  EXPECTED="$(awk -v a="$ARCHIVE_NAME" 'index($0, a) { print $1; exit }' "$SUM_FILENAME")"
  [ -n "$EXPECTED" ] || die "no checksum entry for $ARCHIVE_NAME in $SUM_FILENAME"

  ACTUAL="$(sha256_of "$ARCHIVE_NAME")"
  if [ "$EXPECTED" != "$ACTUAL" ]; then
    err "checksum mismatch for $ARCHIVE_NAME"
    err "  expected: $EXPECTED"
    err "  actual:   $ACTUAL"
    err "refusing to install (possible corruption or tampering)"
    exit 1
  fi
  say "Checksum OK"

  # --- Extract ---------------------------------------------------------------
  say "Extracting archive..."
  tar -xzf "$ARCHIVE_NAME" || die "failed to extract $ARCHIVE_NAME"

  BIN="$(find . -maxdepth 3 -type f -name "$PROGRAM" -print 2>/dev/null | head -n 1)"
  [ -n "$BIN" ] || die "binary '$PROGRAM' not found inside the archive"
  [ -x "$BIN" ] || chmod +x "$BIN"

  # --- Install location ------------------------------------------------------
  if [ -n "${MERCURY_CORTEX_INSTALL_DIR:-}" ]; then
    DEST="$MERCURY_CORTEX_INSTALL_DIR"
  elif [ -w /usr/local/bin ]; then
    DEST="/usr/local/bin"
  else
    DEST="${HOME}/.local/bin"
  fi

  mkdir -p "$DEST" || die "cannot create install directory $DEST"
  [ -w "$DEST" ] || die "install directory $DEST is not writable"

  if [ -x "$DEST/$PROGRAM" ]; then
    say "Replacing existing installation at $DEST/$PROGRAM"
  fi

  # --- Install atomically ----------------------------------------------------
  STAGED="$DEST/${PROGRAM}.new.$$"
  if ! cp -- "$BIN" "$STAGED" 2>/dev/null; then
    err "insufficient permissions to write to $DEST (run with sudo or adjust write access)"
    exit 1
  fi
  chmod 0755 "$STAGED"
  mv -f -- "$STAGED" "$DEST/$PROGRAM" || { rm -f -- "$STAGED"; die "failed to finalize $DEST/$PROGRAM"; }

  # --- Report ----------------------------------------------------------------
  say ""
  say "Successfully installed $PROGRAM to $DEST/$PROGRAM"

  INSTALLED_VERSION="$("$DEST/$PROGRAM" version 2>/dev/null || "$DEST/$PROGRAM" --version 2>/dev/null || true)"
  [ -n "$INSTALLED_VERSION" ] && say "Installed version: $INSTALLED_VERSION"

  case "$DEST" in
    /usr/local/bin)
      say "The command is already on your PATH."
      ;;
    *)
      say "Add $DEST to your PATH if it is not already present:"
      say "  export PATH=\"$DEST:\$PATH\""
      ;;
  esac
}

main "$@"