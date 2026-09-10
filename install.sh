#!/usr/bin/env bash
set -e

REPO="neuralshyam/port-killer"
BIN_NAME="kport"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     PLATFORM="unknown-linux-gnu";;
    Darwin*)    PLATFORM="apple-darwin";;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="pc-windows-msvc";;
    *)          echo "Unsupported OS: ${OS}"; exit 1;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64|amd64)   ARCH_NAME="x86_64";;
    arm64|aarch64)  ARCH_NAME="aarch64";;
    *)              echo "Unsupported Architecture: ${ARCH}"; exit 1;;
esac

TARGET="${ARCH_NAME}-${PLATFORM}"
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v0.2.0"
fi

URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/port-killer-${TARGET}.tar.gz"

echo "⚡ Installing port-killer (${BIN_NAME}) ${LATEST_TAG} for ${TARGET}..."

mkdir -p "${INSTALL_DIR}"
TMP_DIR=$(mktemp -d)

curl -sL "${URL}" -o "${TMP_DIR}/port-killer.tar.gz" || {
    echo "⚠️ Pre-built release not found, falling back to cargo install..."
    if command -v cargo >/dev/null 2>&1; then
        cargo install kport
        echo "✓ Successfully installed via cargo!"
        exit 0
    else
        echo "Error: cargo is required to build from source."
        exit 1
    fi
}

tar -xzf "${TMP_DIR}/port-killer.tar.gz" -C "${TMP_DIR}"
mv "${TMP_DIR}/port-killer" "${INSTALL_DIR}/${BIN_NAME}"
ln -sf "${INSTALL_DIR}/${BIN_NAME}" "${INSTALL_DIR}/port-killer"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/port-killer"

rm -rf "${TMP_DIR}"

echo "✓ Successfully installed ${BIN_NAME} and port-killer to ${INSTALL_DIR}!"
echo ""
echo "Make sure ${INSTALL_DIR} is in your PATH:"
echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
echo ""
echo "Run 'kport' to get started!"
