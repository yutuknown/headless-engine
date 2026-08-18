#!/usr/bin/env bash
set -e

REPO="yutuknown/headless-engine"
INSTALL_DIR="/usr/local/bin"

echo ">>> Detecting OS and Architecture..."
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$OS" = "darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        ASSET="headless-engine-macos-arm64.tar.gz"
    else
        ASSET="headless-engine-macos-x86_64.tar.gz"
    fi
elif [ "$OS" = "linux" ]; then
    if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        ASSET="headless-engine-linux-arm64.tar.gz"
    else
        ASSET="headless-engine-linux-x86_64.tar.gz"
    fi
else
    echo "Unsupported OS: $OS"
    exit 1
fi

echo ">>> Fetching latest release of Headless Engine..."
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo ">>> Downloading $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET"

echo ">>> Extracting binary..."
tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"

echo ">>> Installing to $INSTALL_DIR/headless-engine..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/headless-engine" "$INSTALL_DIR/headless-engine"
else
    sudo mv "$TMP_DIR/headless-engine" "$INSTALL_DIR/headless-engine"
fi
chmod +x "$INSTALL_DIR/headless-engine"

echo ">>> Headless Engine installed successfully!"
headless-engine --help
