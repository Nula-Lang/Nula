#!/bin/bash

# Extended ANSI color codes for vibrant look
RED='\033[1;31m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
PURPLE='\033[1;35m'
CYAN='\033[1;36m'
WHITE='\033[1;37m'
ORANGE='\033[1;38;5;208m'
PINK='\033[1;38;5;199m'
TEAL='\033[1;38;5;51m'
VIOLET='\033[1;38;5;135m'
NC='\033[0m' # No Color

# Unicode spinner (no emojis)
SPINNER=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')

# Function to display spinner
spinner() {
    local pid=$1
    local delay=0.1
    local i=0
    while [ -d /proc/$pid ]; do
        printf "\r${VIOLET}${SPINNER[$i]} ${WHITE}Processing...${NC}"
        ((i = (i + 1) % ${#SPINNER[@]}))
        sleep $delay
    done
    printf "\r"
}

# Function to download files with spinner and color
download_with_spinner() {
    local url=$1
    local output=$2
    local desc=$3
    echo -e "${ORANGE}┌─[DOWNLOAD]──${NC} ${YELLOW}$desc${NC}"
    curl -L --fail --show-error --progress-bar "$url" -o "$output" &
    spinner $!
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}└─[SUCCESS]──${NC} Downloaded ${CYAN}$desc${NC}"
    else
        echo -e "${RED}└─[ERROR]──${NC} Failed to download ${CYAN}$desc${NC}"
        exit 1
    fi
}

# Fancy banner with enhanced borders
echo -e "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
echo -e "${PURPLE}          Nula Programming Language Installer                ${NC}"
echo -e "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
echo

# Create Nula directory in home
echo -e "${PINK}┌─[INFO]──${NC} Creating ~/.nula/lib directory..."
mkdir -p ~/.nula/lib & spinner $!
if [ $? -eq 0 ]; then
    echo -e "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}~/.nula/lib${NC} directory"
else
    echo -e "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}~/.nula/lib${NC} directory"
    exit 1
fi

# Create temporary directory
echo -e "${PINK}┌─[INFO]──${NC} Creating temporary directory..."
mkdir -p /tmp/nula-install & spinner $!
cd /tmp/nula-install
if [ $? -eq 0 ]; then
    echo -e "${GREEN}└─[SUCCESS]──${NC} Created temporary directory"
else
    echo -e "${RED}└─[ERROR]──${NC} Failed to create temporary directory"
    exit 1
fi

# Download files with vibrant colors
download_with_spinner "https://github.com/Nula-Lang/Nula/releases/download/v0.2/nula-zig" "/tmp/nula-install/nula-zig" "Nula Zig binary"
download_with_spinner "https://github.com/Nula-Lang/Nula/releases/download/v0.2/nula-go" "/tmp/nula-install/nula-go" "Nula Go binary"
download_with_spinner "https://github.com/Nula-Lang/Nula/releases/download/v0.2/nula" "/tmp/nula-install/nula" "Nula main binary"
download_with_spinner "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula.png" "/tmp/nula-install/nula.png" "Nula icon"
download_with_spinner "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-terminal.sh" "/tmp/nula-install/nula-terminal.sh" "Nula terminal script"
download_with_spinner "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-lang.desktop" "/tmp/nula-install/nula-lang.desktop" "Nula desktop file"

# Create Nula directory
echo -e "${PINK}┌─[INFO]──${NC} Creating Nula directory..."
sudo mkdir -p /usr/lib/nula & spinner $!
if [ $? -eq 0 ]; then
    echo -e "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}/usr/lib/nula${NC} directory"
else
    echo -e "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}/usr/lib/nula${NC} directory"
    exit 1
fi

# Update file permissions with flair
echo -e "${PINK}┌─[INFO]──${NC} Updating file permissions..."
sudo chmod a+x /tmp/nula-install/nula-terminal.sh & spinner $!
sudo chmod a+x /tmp/nula-install/nula & spinner $!
sudo chmod a+x /tmp/nula-install/nula-go & spinner $!
sudo chmod a+x /tmp/nula-install/nula-zig & spinner $!
echo -e "${GREEN}└─[SUCCESS]──${NC} Permissions updated for all files"

# Move files to system directories
echo -e "${PINK}┌─[INFO]──${NC} Moving files to system directories..."
sudo mv /tmp/nula-install/nula /usr/bin/ & spinner $!
mv /tmp/nula-install/nula-zig ~/.nula/lib/ & spinner $!
mv /tmp/nula-install/nula-go ~/.nula/lib/ & spinner $!
sudo mv /tmp/nula-install/nula-terminal.sh /usr/lib/nula/ & spinner $!
sudo mv /tmp/nula-install/nula.png /usr/share/icons/ & spinner $!
sudo mv /tmp/nula-install/nula-lang.desktop /usr/share/applications/ & spinner $!
echo -e "${GREEN}└─[SUCCESS]──${NC} All files moved to their destinations"

# Clean up
echo -e "${PINK}┌─[INFO]──${NC} Cleaning up temporary files..."
rm -rf /tmp/nula-install & spinner $!
echo -e "${GREEN}└─[SUCCESS]──${NC} Temporary files removed"

# Final message with enhanced borders
echo
echo -e "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
echo -e "${GREEN}          Nula Programming Language Installed!               ${NC}"
echo -e "${CYAN} Run the ${YELLOW}nula${NC} command or launch ${YELLOW}Nula${NC} from your menu.${NC}"
echo -e "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"

# Wait for user to admire the output
echo -e "${ORANGE}┌─[INFO]──${NC} This window will close in 10 seconds..."
sleep 10

# Test Nula installation
echo -e "${PINK}┌─[INFO]──${NC} Testing Nula installation..."
nula --version & spinner $!
if [ $? -eq 0 ]; then
    echo -e "${GREEN}└─[SUCCESS]──${NC} Nula is working perfectly!"
else
    echo -e "${RED}└─[ERROR]──${NC} Nula installation test failed. Please check the setup."
    exit 1
fi

echo -e "${VIOLET}┌─[THANKS]──${NC} Thank you for installing Nula! Happy coding!"
echo -e "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
