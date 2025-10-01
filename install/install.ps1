# Extended ANSI color codes for vibrant look
$RED = "`e[1;31m"
$GREEN = "`e[1;32m"
$YELLOW = "`e[1;33m"
$BLUE = "`e[1;34m"
$PURPLE = "`e[1;35m"
$CYAN = "`e[1;36m"
$WHITE = "`e[1;37m"
$ORANGE = "`e[1;38;5;208m"
$PINK = "`e[1;38;5;199m"
$TEAL = "`e[1;38;5;51m"
$VIOLET = "`e[1;38;5;135m"
$NC = "`e[0m" # No Color

# Unicode spinner (no emojis)
$SPINNER = @('⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏')

# Function to display spinner
function Show-Spinner {
    param (
        [Parameter(Mandatory=$true)]
        [int]$ProcessId
    )
    $delay = 0.1
    $i = 0
    while (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue) {
        Write-Host -NoNewline "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}"
        $i = ($i + 1) % $SPINNER.Length
        Start-Sleep -Seconds $delay
    }
    Write-Host -NoNewline "`r"
}

# Function to download files with spinner and color
function Download-WithSpinner {
    param (
        [string]$Url,
        [string]$Output,
        [string]$Description
    )
    Write-Host "${ORANGE}┌─[DOWNLOAD]──${NC} ${YELLOW}$Description${NC}"
    $process = Start-Process -FilePath "curl" -ArgumentList "-L --fail --show-error --progress-bar $Url -o $Output" -NoNewWindow -PassThru
    Show-Spinner -ProcessId $process.Id
    if ($process.ExitCode -eq 0) {
        Write-Host "${GREEN}└─[SUCCESS]──${NC} Downloaded ${CYAN}$Description${NC}"
    } else {
        Write-Host "${RED}└─[ERROR]──${NC} Failed to download ${CYAN}$Description${NC}"
        exit 1
    }
}

# Fancy banner with enhanced borders
Write-Host "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
Write-Host "${PURPLE} Nula Programming Language Installer ${NC}"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
Write-Host ""

# Ask user for atomic installation option
Write-Host "${PINK}┌─[CONFIG]──${NC} Do you want to install Nula as atomic? (y/n)"
$atomic_choice = Read-Host "Enter your choice (y for true, n for false)"
if ($atomic_choice -match '^[Yy]$') {
    $is_atomic = $true
    Write-Host "${GREEN}└─[INFO]──${NC} Atomic installation selected. Nula binary will be placed in ${TEAL}~/.local/bin/${NC}"
} else {
    $is_atomic = $false
    Write-Host "${GREEN}└─[INFO]──${NC} Standard installation selected. Nula binary will be placed in ${TEAL}/usr/bin/${NC}"
}

# Create Nula directory in home
Write-Host "${PINK}┌─[INFO]──${NC} Creating ~/.nula/lib directory..."
$nulaLibPath = Join-Path $env:HOME ".nula/lib"
New-Item -ItemType Directory -Path $nulaLibPath -Force | Out-Null
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "New-Item -ItemType Directory -Path $nulaLibPath -Force" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}~/.nula/lib${NC} directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}~/.nula/lib${NC} directory"
    exit 1
}

# Create ~/.local/bin if atomic installation is selected
if ($is_atomic) {
    Write-Host "${PINK}┌─[INFO]──${NC} Creating ~/.local/bin directory..."
    $localBinPath = Join-Path $env:HOME ".local/bin"
    New-Item -ItemType Directory -Path $localBinPath -Force | Out-Null
    $process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "New-Item -ItemType Directory -Path $localBinPath -Force" -NoNewWindow -PassThru
    Show-Spinner -ProcessId $process.Id
    if ($process.ExitCode -eq 0) {
        Write-Host "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}~/.local/bin${NC} directory"
    } else {
        Write-Host "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}~/.local/bin${NC} directory"
        exit 1
    }
}

# Create temporary directory
Write-Host "${PINK}┌─[INFO]──${NC} Creating temporary directory..."
$tempDir = "/tmp/nula-install"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "New-Item -ItemType Directory -Path $tempDir -Force" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
Set-Location $tempDir
if ($?) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created temporary directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create temporary directory"
    exit 1
}

# Download files with vibrant colors
Download-WithSpinner -Url "https://github.com/Nula-Lang/Nula/releases/download/v0.3/nula-zig" -Output "$tempDir/nula-zig" -Description "Nula Zig binary"
Download-WithSpinner -Url "https://github.com/Nula-Lang/Nula/releases/download/v0.3/nula-go" -Output "$tempDir/nula-go" -Description "Nula Go binary"
Download-WithSpinner -Url "https://github.com/Nula-Lang/Nula/releases/download/v0.3/nula" -Output "$tempDir/nula" -Description "Nula main binary"
Download-WithSpinner -Url "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula.png" -Output "$tempDir/nula.png" -Description "Nula icon"
Download-WithSpinner -Url "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-terminal.sh" -Output "$tempDir/nula-terminal.sh" -Description "Nula terminal script"
Download-WithSpinner -Url "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-lang.desktop" -Output "$tempDir/nula-lang.desktop" -Description "Nula desktop file"

# Create Nula directory
Write-Host "${PINK}┌─[INFO]──${NC} Creating Nula directory..."
$process = Start-Process -FilePath "sudo" -ArgumentList "mkdir -p /usr/lib/nula" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}/usr/lib/nula${NC} directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}/usr/lib/nula${NC} directory"
    exit 1
}

# Update file permissions with flair
Write-Host "${PINK}┌─[INFO]──${NC} Updating file permissions..."
$process = Start-Process -FilePath "sudo" -ArgumentList "chmod a+x $tempDir/nula-terminal.sh" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
$process = Start-Process -FilePath "sudo" -ArgumentList "chmod a+x $tempDir/nula" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
$process = Start-Process -FilePath "sudo" -ArgumentList "chmod a+x $tempDir/nula-go" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
$process = Start-Process -FilePath "sudo" -ArgumentList "chmod a+x $tempDir/nula-zig" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
Write-Host "${GREEN}└─[SUCCESS]──${NC} Permissions updated for all files"

# Move files to system directories
Write-Host "${PINK}┌─[INFO]──${NC} Moving files to system directories..."
if ($is_atomic) {
    Move-Item -Path "$tempDir/nula" -Destination (Join-Path $env:HOME ".local/bin/") -Force
    $process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path $tempDir/nula -Destination $(Join-Path $env:HOME '.local/bin/') -Force" -NoNewWindow -PassThru
    Show-Spinner -ProcessId $process.Id
} else {
    $process = Start-Process -FilePath "sudo" -ArgumentList "mv $tempDir/nula /usr/bin/" -NoNewWindow -PassThru
    Show-Spinner -ProcessId $process.Id
}
Move-Item -Path "$tempDir/nula-zig" -Destination (Join-Path $env:HOME ".nula/lib/") -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path $tempDir/nula-zig -Destination $(Join-Path $env:HOME '.nula/lib/') -Force" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
Move-Item -Path "$tempDir/nula-go" -Destination (Join-Path $env:HOME ".nula/lib/") -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path $tempDir/nula-go -Destination $(Join-Path $env:HOME '.nula/lib/') -Force" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
$process = Start-Process -FilePath "sudo" -ArgumentList "mv $tempDir/nula-terminal.sh /usr/lib/nula/" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
$process = Start-Process -FilePath "sudo" -ArgumentList "mv $tempDir/nula.png /usr/share/icons/" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
$process = Start-Process -FilePath "sudo" -ArgumentList "mv $tempDir/nula-lang.desktop /usr/share/applications/" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
Write-Host "${GREEN}└─[SUCCESS]──${NC} All files moved to their destinations"

# Clean up
Write-Host "${PINK}┌─[INFO]──${NC} Cleaning up temporary files..."
Remove-Item -Path $tempDir -Recurse -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Remove-Item -Path $tempDir -Recurse -Force" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
Write-Host "${GREEN}└─[SUCCESS]──${NC} Temporary files removed"

# Final message with enhanced borders
Write-Host ""
Write-Host "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
Write-Host "${GREEN} Nula Programming Language Installed! ${NC}"
Write-Host "${CYAN} Run the ${YELLOW}nula${NC} command or launch ${YELLOW}Nula${NC} from your menu.${NC}"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"

# Wait for user to admire the output
Write-Host "${ORANGE}┌─[INFO]──${NC} This window will close in 10 seconds..."
Start-Sleep -Seconds 10

# Test Nula installation
Write-Host "${PINK}┌─[INFO]──${NC} Testing Nula installation..."
$process = Start-Process -FilePath "nula" -ArgumentList "--version" -NoNewWindow -PassThru
Show-Spinner -ProcessId $process.Id
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Nula is working perfectly!"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Nula installation test failed. Please check the setup."
    exit 1
}

Write-Host "${VIOLET}┌─[THANKS]──${NC} Thank you for installing Nula! Happy coding!"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
