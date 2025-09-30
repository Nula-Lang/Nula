# Extended ANSI color codes for vibrant look (PowerShell compatible)
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
        [System.Diagnostics.Process]$Process
    )
    $delay = 0.1
    $i = 0
    while (-not $Process.HasExited) {
        Write-Host "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}" -NoNewline
        $i = ($i + 1) % $SPINNER.Length
        Start-Sleep -Milliseconds ($delay * 1000)
    }
    Write-Host "`r" -NoNewline
}

# Function to download files with spinner and color
function Download-WithSpinner {
    param (
        [string]$Url,
        [string]$Output,
        [string]$Desc
    )
    Write-Host "${ORANGE}┌─[DOWNLOAD]──${NC} ${YELLOW}$Desc${NC}"
    $process = Start-Process -FilePath "curl" -ArgumentList "-L", "--fail", "--show-error", "--progress-bar", $Url, "-o", $Output -NoNewWindow -PassThru
    Show-Spinner -Process $process
    if ($process.ExitCode -eq 0) {
        Write-Host "${GREEN}└─[SUCCESS]──${NC} Downloaded ${CYAN}$Desc${NC}"
    } else {
        Write-Host "${RED}└─[ERROR]──${NC} Failed to download ${CYAN}$Desc${NC}"
        exit 1
    }
}

# Fancy banner with enhanced borders
Write-Host "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
Write-Host "${PURPLE}          Nula Programming Language Installer                ${NC}"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
Write-Host ""

# Create Nula directory in home
Write-Host "${PINK}┌─[INFO]──${NC} Creating ~/.nula/lib directory..."
New-Item -ItemType Directory -Path "$env:USERPROFILE\.nula\lib" -Force | Out-Null
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "New-Item -ItemType Directory -Path '$env:USERPROFILE\.nula\lib' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}$env:USERPROFILE\.nula\lib${NC} directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}$env:USERPROFILE\.nula\lib${NC} directory"
    exit 1
}

# Create temporary directory
Write-Host "${PINK}┌─[INFO]──${NC} Creating temporary directory..."
New-Item -ItemType Directory -Path "$env:TEMP\nula-install" -Force | Out-Null
Set-Location "$env:TEMP\nula-install"
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "New-Item -ItemType Directory -Path '$env:TEMP\nula-install' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created temporary directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create temporary directory"
    exit 1
}

# Download files with vibrant colors
Download-WithSpinner "https://github.com/Nula-Lang/Nula/releases/download/v0.3/nula-zig" "$env:TEMP\nula-install\nula-zig" "Nula Zig binary"
Download-WithSpinner "https://github.com/Nula-Lang/Nula/releases/download/v0.3/nula-go" "$env:TEMP\nula-install\nula-go" "Nula Go binary"
Download-WithSpinner "https://github.com/Nula-Lang/Nula/releases/download/v0.3/nula" "$env:TEMP\nula-install\nula" "Nula main binary"
Download-WithSpinner "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula.png" "$env:TEMP\nula-install\nula.png" "Nula icon"
Download-WithSpinner "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-terminal.sh" "$env:TEMP\nula-install\nula-terminal.sh" "Nula terminal script"
Download-WithSpinner "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-lang.desktop" "$env:TEMP\nula-install\nula-lang.desktop" "Nula desktop file"

# Create Nula directory in Program Files
Write-Host "${PINK}┌─[INFO]──${NC} Creating Nula directory..."
New-Item -ItemType Directory -Path "$env:ProgramFiles\nula" -Force | Out-Null
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "New-Item -ItemType Directory -Path '$env:ProgramFiles\nula' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}$env:ProgramFiles\nula${NC} directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}$env:ProgramFiles\nula${NC} directory"
    exit 1
}

# Update file permissions (PowerShell equivalent: setting execution attributes)
Write-Host "${PINK}┌─[INFO]──${NC} Updating file permissions..."
$files = @("$env:TEMP\nula-install\nula-terminal.sh", "$env:TEMP\nula-install\nula", "$env:TEMP\nula-install\nula-go", "$env:TEMP\nula-install\nula-zig")
foreach ($file in $files) {
    $process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Set-ItemProperty -Path '$file' -Name IsReadOnly -Value `$false" -NoNewWindow -PassThru
    Show-Spinner -Process $process
}
Write-Host "${GREEN}└─[SUCCESS]──${NC} Permissions updated for all files"

# Move files to system directories
Write-Host "${PINK}┌─[INFO]──${NC} Moving files to system directories..."
Move-Item -Path "$env:TEMP\nula-install\nula" -Destination "$env:ProgramFiles\nula\nula" -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path '$env:TEMP\nula-install\nula' -Destination '$env:ProgramFiles\nula\nula' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process

Move-Item -Path "$env:TEMP\nula-install\nula-zig" -Destination "$env:USERPROFILE\.nula\lib\nula-zig" -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path '$env:TEMP\nula-install\nula-zig' -Destination '$env:USERPROFILE\.nula\lib\nula-zig' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process

Move-Item -Path "$env:TEMP\nula-install\nula-go" -Destination "$env:USERPROFILE\.nula\lib\nula-go" -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path '$env:TEMP\nula-install\nula-go' -Destination '$env:USERPROFILE\.nula\lib\nula-go' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process

Move-Item -Path "$env:TEMP\nula-install\nula-terminal.sh" -Destination "$env:ProgramFiles\nula\nula-terminal.sh" -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path '$env:TEMP\nula-install\nula-terminal.sh' -Destination '$env:ProgramFiles\nula\nula-terminal.sh' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process

Move-Item -Path "$env:TEMP\nula-install\nula.png" -Destination "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\nula.png" -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path '$env:TEMP\nula-install\nula.png' -Destination '$env:APPDATA\Microsoft\Windows\Start Menu\Programs\nula.png' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process

Move-Item -Path "$env:TEMP\nula-install\nula-lang.desktop" -Destination "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\nula-lang.desktop" -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Move-Item -Path '$env:TEMP\nula-install\nula-lang.desktop' -Destination '$env:APPDATA\Microsoft\Windows\Start Menu\Programs\nula-lang.desktop' -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process

Write-Host "${GREEN}└─[SUCCESS]──${NC} All files moved to their destinations"

# Clean up
Write-Host "${PINK}┌─[INFO]──${NC} Cleaning up temporary files..."
Remove-Item -Path "$env:TEMP\nula-install" -Recurse -Force
$process = Start-Process -FilePath "powershell" -ArgumentList "-Command", "Remove-Item -Path '$env:TEMP\nula-install' -Recurse -Force" -NoNewWindow -PassThru
Show-Spinner -Process $process
Write-Host "${GREEN}└─[SUCCESS]──${NC} Temporary files removed"

# Final message with enhanced borders
Write-Host ""
Write-Host "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
Write-Host "${GREEN}          Nula Programming Language Installed!               ${NC}"
Write-Host "${CYAN} Run the ${YELLOW}nula${NC} command or launch ${YELLOW}Nula${NC} from your menu.${NC}"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"

# Wait for user to admire the output
Write-Host "${ORANGE}┌─[INFO]──${NC} This window will close in 10 seconds..."
Start-Sleep -Seconds 10

# Test Nula installation
Write-Host "${PINK}┌─[INFO]──${NC} Testing Nula installation..."
$process = Start-Process -FilePath "$env:ProgramFiles\nula\nula" -ArgumentList "--version" -NoNewWindow -PassThru
Show-Spinner -Process $process
if ($process.ExitCode -eq 0) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Nula is working perfectly!"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Nula installation test failed. Please check the setup."
    exit 1
}

Write-Host "${VIOLET}┌─[THANKS]──${NC} Thank you for installing Nula! Happy coding!"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
