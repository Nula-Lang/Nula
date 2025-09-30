#!/bin/bash

set -e

echo "[INFO] Checking distribution..."

# Detect package manager
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    DISTRO=$(uname -s)
fi

echo "[INFO] Distribution detected: $DISTRO"

# Function to install dependencies
install_dep() {
    dep=$1
    case "$DISTRO" in
        hackeros|ubuntu|debian)
            sudo apt update && sudo apt install -y $dep
            ;;
        fedora)
            sudo dnf install -y $dep
            ;;
        fedora-silverblue)
            echo "[INFO] Use rpm-ostree to install $dep manually."
            ;;
        zenit)
            sudo zpm install-apt $dep
            ;;
        opensuse|suse)
            sudo zypper install -y $dep
            ;;
        arch|manjaro)
            sudo pacman -Syu --noconfirm $dep
            ;;
        gentoo)
            sudo emerge $dep
            ;;
        slackware)
            echo "[INFO] Install $dep manually using slackpkg or compile from source."
            ;;
        Darwin)
            brew install $dep
            ;;
        *)
            echo "[WARN] Unknown distribution. Please install $dep manually."
            ;;
    esac
}

# Required dependencies
deps=(go zig gcc)

# Check and install dependencies
echo "[INFO] Checking and installing dependencies..."
for dep in "${deps[@]}"; do
    if ! command -v $dep &> /dev/null; then
        echo "[INFO] $dep not found, installing..."
        install_dep $dep
    else
        echo "[INFO] $dep is already installed."
    fi
done

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "[INFO] Rust not found, installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source $HOME/.cargo/env
else
    echo "[INFO] Rust is already installed."
fi

# Clone repository
echo "[RUN] Cloning the repository..."
git clone https://github.com/Nula-Lang/Nula.git /tmp/Nula
cd /tmp/Nula

# Build Nula

echo "[RUN] Building nula (Go)..."
cd /tmp/Nula/nula/go/
if [ ! -f go.mod ]; then
    go mod init example.com/m/v2
fi
go mod tidy
go build
sudo mv m nula-go
sudo chmod a+x nula-go
sudo mv /tmp/Nula/nula/nula-go /usr/bin/
cd /tmp/Nula/nula/zig/

echo "[RUN] Building nula (Zig)..."
zig build-exe main.zig -O ReleaseFast
sudo mv main nula-zig
sudo chmod a+x nula-zig
sudo mv tmp/Nula/nula/nula-zig /usr/bin/

cd /tmp/Nula/nula/
echo "[RUN] Building nula (Rust)..."
cargo build --release
cd /tmp/Nula/nula/target/release/
sudo chmod a+x nula
sudo mv /tmp/Nula/nula/target/release/nula /usr/bin/

echo "[INFO] The operation has been completed."
echo "[PLEASE] Run the nula command or launch the application from the nula program menu."
