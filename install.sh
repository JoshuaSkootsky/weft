#!/bin/bash
set -euo pipefail

REPO="weft-vcs/weft"
VERSION="v0.1.0"

echo "Installing WEFT ${VERSION}..."

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        echo "Supported architectures: x86_64, aarch64, arm64"
        exit 1
        ;;
esac

case "$OS" in
    linux)
        TARGET="${ARCH}-unknown-linux-gnu"
        ;;
    darwin)
        TARGET="${ARCH}-apple-darwin"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        echo "Supported systems: linux, darwin"
        exit 1
        ;;
esac

BINARY_URL="https://github.com/${REPO}/releases/download/${VERSION}/weft-${TARGET}"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_PATH="${INSTALL_DIR}/weft"

FORCE_INSTALL="${FORCE_INSTALL:-false}"

if [ -f /usr/local/bin/weft ] && [ "$FORCE_INSTALL" != "true" ]; then
    echo "WEFT already installed at /usr/local/bin/weft"
    echo "To reinstall, run: FORCE_INSTALL=true $0"
    exit 0
fi

if [ -f "${BINARY_PATH}" ] && [ "$FORCE_INSTALL" != "true" ]; then
    echo "WEFT already installed at ${BINARY_PATH}"
    echo "To reinstall, run: FORCE_INSTALL=true $0"
    exit 0
fi

echo "Downloading WEFT ${TARGET} from ${BINARY_URL}..."

mkdir -p "${INSTALL_DIR}"

if ! curl -sSL --fail-with-body "${BINARY_URL}" -o "${BINARY_PATH}"; then
    echo "Error: Failed to download WEFT binary"
    echo "The binary may not be available for your platform yet."
    echo "Build from source with: cargo install --path ."
    exit 1
fi

chmod 755 "${BINARY_PATH}"

echo ""
echo "✓ WEFT installed to ${BINARY_PATH}"
echo ""

if command -v jj &> /dev/null; then
    JJ_VERSION=$(jj --version 2>/dev/null || echo "unknown")
    echo "✓ jj found (version ${JJ_VERSION})"
    echo ""
    echo "Next steps:"
    echo "  cd your-git-repo"
    echo "  weft init"
    echo "  weft save \"checkpoint message\""
else
    echo "⚠️  jj not found. WEFT requires jj (Jujutsu) v0.15+ to work."
    echo ""
    echo "Install jj:"
    echo "  macOS: brew install jj"
    echo "  Linux: curl -sSL https://github.com/martinvonz/jj/releases/download/v0.15.1/jj-v0.15.1-${TARGET}.tar.gz | tar -xz && sudo mv jj /usr/local/bin/"
    echo "  Cargo: cargo install jj-cli"
    echo ""
    echo "After installing jj, run 'weft init' in your git repository."
fi
