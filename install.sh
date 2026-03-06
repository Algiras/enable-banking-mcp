#!/usr/bin/env sh
set -e

REPO="Algiras/enable-banking-mcp"
BIN_NAME="enable-banking-mcp"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ── Detect OS and architecture ───────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  ARTIFACT="${BIN_NAME}-linux-x86_64" ;;
      aarch64) ARTIFACT="${BIN_NAME}-linux-aarch64" ;;
      arm64)   ARTIFACT="${BIN_NAME}-linux-aarch64" ;;
      *)       echo "Unsupported Linux architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  ARTIFACT="${BIN_NAME}-macos-x86_64" ;;
      arm64)   ARTIFACT="${BIN_NAME}-macos-aarch64" ;;
      *)       echo "Unsupported macOS architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "For Windows, download the .exe from https://github.com/${REPO}/releases/latest"
    exit 1
    ;;
esac

# ── Resolve latest release tag ───────────────────────────────────────────────
echo "Fetching latest release..."
if command -v curl >/dev/null 2>&1; then
  FETCH="curl -fsSL"
elif command -v wget >/dev/null 2>&1; then
  FETCH="wget -qO-"
else
  echo "Error: curl or wget is required" && exit 1
fi

LATEST_TAG=$(
  $FETCH "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
)

if [ -z "$LATEST_TAG" ]; then
  echo "Error: could not determine latest release tag"
  exit 1
fi

echo "Latest version: $LATEST_TAG"

# ── Download ─────────────────────────────────────────────────────────────────
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARTIFACT}"
TMP_FILE="$(mktemp)"

echo "Downloading ${ARTIFACT}..."
$FETCH "$DOWNLOAD_URL" -o "$TMP_FILE" 2>/dev/null || {
  # wget variant
  $FETCH "$DOWNLOAD_URL" > "$TMP_FILE"
}

chmod +x "$TMP_FILE"

# ── Install ───────────────────────────────────────────────────────────────────
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_FILE" "${INSTALL_DIR}/${BIN_NAME}"
else
  echo "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "$TMP_FILE" "${INSTALL_DIR}/${BIN_NAME}"
fi

echo ""
echo "✅ enable-banking-mcp ${LATEST_TAG} installed to ${INSTALL_DIR}/${BIN_NAME}"
echo ""
echo "Next steps:"
echo "  Sandbox:    ${BIN_NAME} configure && ${BIN_NAME} init && ${BIN_NAME} install"
echo "  Production: ${BIN_NAME} register  && ${BIN_NAME} init && ${BIN_NAME} install"
echo ""
echo "Docs: https://github.com/${REPO}#readme"
