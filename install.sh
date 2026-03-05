#!/bin/sh
set -e

# Default to latest release
VERSION=$(curl -sL https://api.github.com/repos/your-username/depg/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "Error: Could not retrieve latest version."
    exit 1
fi

echo "Installing depg $VERSION"

OS=$(uname -s)
ARCH=$(uname -m)

if [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET="x86_64-unknown-linux-musl"
    else
        echo "Error: Architecture $ARCH on Linux is unsupported."
        exit 1
    fi
elif [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET="x86_64-apple-darwin"
    elif [ "$ARCH" = "arm64" ]; then
        TARGET="aarch64-apple-darwin"
    else
        echo "Error: Architecture $ARCH on macOS is unsupported."
        exit 1
    fi
else
    echo "Error: OS $OS is unsupported."
    exit 1
fi

URL="https://github.com/your-username/depg/releases/download/$VERSION/depg-$TARGET.tar.gz"

echo "Downloading from $URL"
curl -sL "$URL" | tar -xz

# Move binary to a directory in PATH
if [ -d "$HOME/.local/bin" ]; then
    mv depg "$HOME/.local/bin/"
    echo "Successfully installed depg to $HOME/.local/bin/"
    echo "Make sure $HOME/.local/bin is in your PATH."
else
    sudo mv depg /usr/local/bin/
    echo "Successfully installed depg to /usr/local/bin/"
fi
