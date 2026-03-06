#!/usr/bin/env sh
set -e

REPO="Algiras/enable-banking-mcp"
BIN_NAME="enable-banking-mcp"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"
GITHUB_DL="https://github.com/${REPO}/releases/download"

# ── Helpers ───────────────────────────────────────────────────────────────────
say()  { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
err()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || err "'$1' is required but not installed."; }

need curl

# ── Detect platform ───────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)         ARTIFACT="${BIN_NAME}-linux-x86_64" ;;
      aarch64|arm64)  ARTIFACT="${BIN_NAME}-linux-aarch64" ;;
      *)              err "Unsupported Linux architecture: $ARCH" ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  ARTIFACT="${BIN_NAME}-macos-x86_64" ;;
      arm64)   ARTIFACT="${BIN_NAME}-macos-aarch64" ;;
      *)       err "Unsupported macOS architecture: $ARCH" ;;
    esac
    ;;
  *)
    err "Unsupported OS: $OS. For Windows download from https://github.com/${REPO}/releases/latest"
    ;;
esac

say "Detected platform: $OS/$ARCH → $ARTIFACT"

# ── Resolve latest tag ────────────────────────────────────────────────────────
say "Fetching latest release..."
TAG=$(curl -fsSL "$GITHUB_API" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
[ -n "$TAG" ] || err "Could not determine latest release tag. Check your internet connection."
say "Latest release: $TAG"

# ── Download binary + checksum ────────────────────────────────────────────────
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BIN_URL="${GITHUB_DL}/${TAG}/${ARTIFACT}"
SUM_URL="${GITHUB_DL}/${TAG}/checksums.txt"
TMP_BIN="${TMP_DIR}/${BIN_NAME}"
TMP_SUM="${TMP_DIR}/checksums.txt"

say "Downloading ${ARTIFACT}..."
curl -fSL --progress-bar "$BIN_URL" -o "$TMP_BIN"

say "Verifying checksum..."
curl -fsSL "$SUM_URL" -o "$TMP_SUM"

# Extract expected checksum for our artifact
EXPECTED=$(grep " ${ARTIFACT}$" "$TMP_SUM" | awk '{print $1}')
[ -n "$EXPECTED" ] || err "Checksum not found for ${ARTIFACT} in checksums.txt"

# Verify (shasum on macOS, sha256sum on Linux)
if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL=$(sha256sum "$TMP_BIN" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL=$(shasum -a 256 "$TMP_BIN" | awk '{print $1}')
else
  say "Warning: no sha256 tool found, skipping checksum verification"
  ACTUAL="$EXPECTED"
fi

[ "$ACTUAL" = "$EXPECTED" ] || err "Checksum mismatch!\n  expected: $EXPECTED\n  got:      $ACTUAL"
say "Checksum OK ✓"

chmod +x "$TMP_BIN"

# ── Install ───────────────────────────────────────────────────────────────────
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_BIN" "${INSTALL_DIR}/${BIN_NAME}"
else
  say "Installing to ${INSTALL_DIR} (sudo required)..."
  sudo mv "$TMP_BIN" "${INSTALL_DIR}/${BIN_NAME}"
fi

INSTALLED_VERSION=$("${INSTALL_DIR}/${BIN_NAME}" --version 2>/dev/null || echo "$TAG")

say "✅ ${BIN_NAME} ${INSTALLED_VERSION} installed → ${INSTALL_DIR}/${BIN_NAME}"
echo ""
echo "  Next steps:"
echo ""
echo "  Sandbox (testing):    ${BIN_NAME} configure && ${BIN_NAME} init && ${BIN_NAME} install"
echo "  Production (real bank): ${BIN_NAME} register  && ${BIN_NAME} init && ${BIN_NAME} install"
echo ""
echo "  Docs: https://github.com/${REPO}#readme"
