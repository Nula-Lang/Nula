#!/bin/bash
set -e

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Spinner function
spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
    while kill -0 "$pid" 2>/dev/null; do
        for i in $(seq 0 9); do
            printf "\r${CYAN}${spinstr:$i:1}${NC} $2"
            sleep $delay
        done
    done
    printf "\r${GREEN}✓${NC} $2\n"
}

# Run command with spinner in background
run_with_spinner() {
    local cmd="$1"
    local msg="$2"
    bash -c "$cmd" &
    local pid=$!
    spinner $pid "$msg"
    wait $pid 2>/dev/null
}

echo -e "${BLUE}[INFO] Checking distribution...${NC}"

# Detect package manager and distribution details
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$(echo "$ID" | tr '[:upper:]' '[:lower:]')
    if [ -n "$VARIANT_ID" ]; then
        DISTRO="$DISTRO-$VARIANT_ID"
    fi
elif [ -f /etc/lsb-release ]; then
    . /etc/lsb-release
    DISTRO=$(echo "$DISTRIB_ID" | tr '[:upper:]' '[:lower:]')
else
    DISTRO=$(uname -s | tr '[:upper:]' '[:lower:]')
fi

echo -e "${GREEN}[INFO] Distribution detected: $DISTRO${NC}"

# Determine if this is an atomic/immutable distribution
is_atomic=false
case "$DISTRO" in
    fedora-silverblue|fedora-kinoite|opensuse-microos|steamdeck|bottlesos|nixos|guix)
        is_atomic=true
        echo -e "${YELLOW}[INFO] Atomic/immutable distribution detected. Binaries will be placed in user-local paths.${NC}"
        ;;
esac

# Function to install dependencies
install_dep() {
    local dep=$1
    local pkg=$dep
    case $dep in
        go)
            case "$DISTRO" in
                ubuntu|debian|hackeros|zenit|kali|linuxmint|pop|elementary|zorinos|mx|deepin|parrot|raspbian|devuan|antix|peppermint|sparky)
                    pkg="golang-go"
                    ;;
                arch|manjaro|endeavouros|kaos|alpine|freebsd|openbsd|netbsd|dragonfly|mageia|crux|slitaz|tinycore|artix|garuda|cachyos)
                    pkg="go"
                    ;;
                *)
                    pkg="golang"
                    ;;
            esac
            ;;
        zig)
            pkg="zig"
            ;;
        gcc)
            pkg="gcc"
            ;;
        git)
            pkg="git"
            ;;
        curl)
            pkg="curl"
            ;;
    esac
    case "$DISTRO" in
        ubuntu|debian|hackeros|kali|linuxmint|pop|elementary|zorinos|mx|deepin|parrot|raspbian|devuan|antix|peppermint|sparky)
            run_with_spinner "sudo apt update && sudo apt install -y $pkg" "Installing $dep on $DISTRO..."
            ;;
        fedora|centos|rhel|almalinux|rockylinux|amzn|oraclelinux|nobara|ultramarine)
            if command -v dnf &> /dev/null; then
                run_with_spinner "sudo dnf install -y $pkg" "Installing $dep on $DISTRO..."
            else
                run_with_spinner "sudo yum install -y $pkg" "Installing $dep on $DISTRO..."
            fi
            ;;
        fedora-silverblue|fedora-kinoite)
            echo -e "${YELLOW}[INFO] On atomic Fedora variants like Silverblue/Kinoite, use rpm-ostree install $pkg and reboot, or layer it manually.${NC}"
            echo -e "${YELLOW}[INFO] For development tools, consider using toolbox or distrobox.${NC}"
            ;;
        zenit)
            run_with_spinner "sudo zpm install-apt $pkg" "Installing $dep on $DISTRO..."
            ;;
        opensuse*|suse|opensuse-microos)
            run_with_spinner "sudo zypper install -y $pkg" "Installing $dep on $DISTRO..."
            if [ "$DISTRO" = "opensuse-microos" ]; then
                echo -e "${YELLOW}[INFO] On MicroOS, consider using transactional-update for persistent changes.${NC}"
            fi
            ;;
        arch|manjaro|endeavouros|kaos|artix|garuda|cachyos)
            run_with_spinner "sudo pacman -Syu --noconfirm $pkg" "Installing $dep on $DISTRO..."
            ;;
        gentoo|sabayon|funtoo|calculate|chromeos)
            run_with_spinner "sudo emerge $pkg" "Installing $dep on $DISTRO..."
            ;;
        slackware)
            echo -e "${YELLOW}[INFO] Install $pkg manually using slackpkg or compile from source on $DISTRO.${NC}"
            ;;
        alpine)
            run_with_spinner "sudo apk update && sudo apk add $pkg" "Installing $dep on $DISTRO..."
            ;;
        solus)
            run_with_spinner "sudo eopkg install -y $pkg" "Installing $dep on $DISTRO..."
            ;;
        void)
            run_with_spinner "sudo xbps-install -Sy $pkg" "Installing $dep on $DISTRO..."
            ;;
        nixos)
            echo -e "${YELLOW}[INFO] On NixOS, use nix-env -iA nixpkgs.$pkg or add to configuration.nix and nixos-rebuild switch.${NC}"
            ;;
        guix)
            echo -e "${YELLOW}[INFO] On Guix, use guix install $pkg.${NC}"
            ;;
        freebsd|dragonfly)
            run_with_spinner "sudo pkg install -y $pkg" "Installing $dep on $DISTRO..."
            ;;
        openbsd)
            run_with_spinner "sudo pkg_add $pkg" "Installing $dep on $DISTRO..."
            ;;
        netbsd)
            run_with_spinner "sudo pkgin install $pkg" "Installing $dep on $DISTRO..."
            ;;
        darwin|macos)
            if ! command -v brew &> /dev/null; then
                echo -e "${CYAN}[INFO] Installing Homebrew...${NC}"
                run_with_spinner "/bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\"" "Installing Homebrew..."
            fi
            run_with_spinner "brew install $pkg" "Installing $dep on $DISTRO..."
            ;;
        mageia)
            run_with_spinner "sudo urpmi --auto $pkg" "Installing $dep on $DISTRO..."
            ;;
        pclinuxos)
            run_with_spinner "sudo apt-get update && sudo apt-get install -y $pkg" "Installing $dep on $DISTRO..."
            ;;
        crux)
            run_with_spinner "sudo prt-get depinst $pkg" "Installing $dep on $DISTRO..."
            ;;
        slitaz)
            run_with_spinner "sudo tazpkg get-install $pkg" "Installing $dep on $DISTRO..."
            ;;
        tinycore)
            run_with_spinner "tce-load -wi $pkg.tcz" "Installing $dep on $DISTRO..."
            ;;
        clear-linux-os)
            echo -e "${YELLOW}[INFO] For Clear Linux, use swupd bundle-add devpkg-$dep or similar manually.${NC}"
            ;;
        openindiana|illumos|solaris)
            run_with_spinner "sudo pkg install $pkg" "Installing $dep on $DISTRO..."
            ;;
        haiku)
            run_with_spinner "pkgman install $pkg" "Installing $dep on $DISTRO..."
            ;;
        minix)
            echo -e "${YELLOW}[INFO] On Minix, use pkgin install $pkg or compile from source manually.${NC}"
            ;;
        *)
            echo -e "${RED}[WARN] Unknown distribution: $DISTRO. Please install $pkg manually.${NC}"
            ;;
    esac
}

# Required dependencies
deps=(git curl go zig gcc)

# Check and install dependencies
echo -e "${BLUE}[INFO] Checking and installing dependencies...${NC}"
for dep in "${deps[@]}"; do
    if ! command -v $dep &> /dev/null; then
        echo -e "${YELLOW}[INFO] $dep not found, installing...${NC}"
        install_dep $dep
    else
        echo -e "${GREEN}[INFO] $dep is already installed.${NC}"
    fi
done

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${YELLOW}[INFO] Rust not found, installing via rustup...${NC}"
    run_with_spinner "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable" "Installing Rust..."
    source "$HOME/.cargo/env"
else
    echo -e "${GREEN}[INFO] Rust is already installed.${NC}"
fi

# Clone repository
echo -e "${BLUE}[RUN] Cloning the repository...${NC}"
run_with_spinner "git clone https://github.com/Nula-Lang/Nula.git /tmp/Nula" "Cloning Nula repository..."
cd /tmp/Nula

# Create user-local directories without sudo
mkdir -p ~/.nula/lib/

# Build Nula (Go)
echo -e "${BLUE}[RUN] Building nula (Go)...${NC}"
cd /tmp/Nula/nula/go/
if [ ! -f go.mod ]; then
    run_with_spinner "go mod init example.com/m/v2" "Initializing Go module..."
fi
run_with_spinner "go mod tidy" "Tidying Go modules..."
run_with_spinner "go build" "Building nula (Go)..."
mv m nula-go
chmod +x nula-go
mv nula-go ~/.nula/lib/

# Build Nula (Zig)
cd /tmp/Nula/nula/zig/
echo -e "${BLUE}[RUN] Building nula (Zig)...${NC}"
run_with_spinner "zig build-exe main.zig -O ReleaseFast" "Building nula (Zig)..."
mv main nula-zig
chmod +x nula-zig
mv nula-zig ~/.nula/lib/

# Build Nula (Rust)
cd /tmp/Nula/nula/
echo -e "${BLUE}[RUN] Building nula (Rust)...${NC}"
run_with_spinner "cargo build --release" "Building nula (Rust)..."
cd target/release/
chmod +x nula

# Install the Rust binary based on distribution type
if $is_atomic; then
    mkdir -p ~/.local/bin/
    mv nula ~/.local/bin/
    echo -e "${GREEN}[INFO] Installed nula binary to ~/.local/bin/ for atomic distribution.${NC}"
    echo -e "${CYAN}[PLEASE] Ensure ~/.local/bin/ is in your PATH. Run 'nula' from there.${NC}"
else
    sudo mv nula /usr/bin/
    echo -e "${GREEN}[INFO] Installed nula binary to /usr/bin/.${NC}"
fi

echo -e "${GREEN}[INFO] The operation has been completed successfully!${NC}"
echo -e "${CYAN}[PLEASE] Run the nula command or launch the application from the nula program menu.${NC}"
