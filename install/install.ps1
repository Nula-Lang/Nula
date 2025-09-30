# PowerShell script for installing Nula Programming Language on Windows

# Define ANSI color codes for vibrant output (Windows 10+ supports ANSI in PowerShell)
$RED = "`e[1;31m"
$GREEN = "`e[1;32m"
$YELLOW = "`e[1;33m"
$BLUE = "`e[1;34m"
$PURPLE = "`e[1;35m"
$CYAN = "`e[1;36m"
$WHITE = "`e[1;37m"
$ORANGE = "`e[38;5;208m"
$PINK = "`e[38;5;199m"
$TEAL = "`e[38;5;51m"
$VIOLET = "`e[38;5;135m"
$NC = "`e[0m"

# Spinner animation (Unicode characters)
$SPINNER = @('⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏')

# Function to display spinner
function Show-Spinner {
    param (
        [scriptblock]$Command,
        [string]$Description
    )
    Write-Host "${ORANGE}┌─[DOWNLOAD]──${NC} ${YELLOW}$Description${NC}"
    $job = Start-Job -ScriptBlock $Command
    $i = 0
    while ($job.State -eq "Running") {
        Write-Host -NoNewline "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}"
        $i = ($i + 1) % $SPINNER.Length
        Start-Sleep -Milliseconds 100
    }
    Write-Host "`r" -NoNewline
    $result = Receive-Job -Job $job -Wait
    if ($job.State -eq "Completed" -and $result.Success) {
        Write-Host "${GREEN}└─[SUCCESS]──${NC} Downloaded ${CYAN}$Description${NC}"
    } else {
        Write-Host "${RED}└─[ERROR]──${NC} Failed to download ${CYAN}$Description${NC}"
        exit 1
    }
}

# Fancy banner
Write-Host "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
Write-Host "${PURPLE}          Nula Programming Language Installer                ${NC}"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
Write-Host ""

# Create temporary directory
Write-Host "${PINK}┌─[INFO]──${NC} Creating temporary directory..."
$tempDir = "$env:TEMP\nula-install"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
Set-Location $tempDir
if ($?) {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created temporary directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create temporary directory"
    exit 1
}

# Download files with spinner
$downloads = @(
    @{Url="https://github.com/Nula-Lang/Nula/releases/download/v0.2/nula-zig"; Path="$tempDir\nula-zig.exe"; Desc="Nula Zig binary"},
    @{Url="https://github.com/Nula-Lang/Nula/releases/download/v0.2/nula-go"; Path="$tempDir\nula-go.exe"; Desc="Nula Go binary"},
    @{Url="https://github.com/Nula-Lang/Nula/releases/download/v0.2/nula"; Path="$tempDir\nula.exe"; Desc="Nula main binary"},
    @{Url="https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula.png"; Path="$tempDir\nula.png"; Desc="Nula icon"},
    @{Url="https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-terminal.sh"; Path="$tempDir\nula-terminal.sh"; Desc="Nula terminal script"},
    @{Url="https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula-lang.desktop"; Path="$tempDir\nula-lang.desktop"; Desc="Nula desktop file"}
)

foreach ($download in $downloads) {
    Show-Spinner -Command {
        $wc = New-Object System.Net.WebClient
        try {
            $wc.DownloadFile($download.Url, $download.Path)
            @{Success=$true}
        } catch {
            @{Success=$false}
        }
    } -Description $download.Desc
}

# Create Nula directory
Write-Host "${PINK}┌─[INFO]──${NC} Creating Nula directory..."
$nulaDir = "C:\Program Files\Nula"
New-Item -ItemType Directory -Path $nulaDir -Force | Out-Null
$job = Start-Job { New-Item -ItemType Directory -Path $using:nulaDir -Force }
$i = 0
while ($job.State -eq "Running") {
    Write-Host -NoNewline "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}"
    $i = ($i + 1) % $SPINNER.Length
    Start-Sleep -Milliseconds 100
}
Write-Host "`r" -NoNewline
if ($job.State -eq "Completed") {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Created ${TEAL}$nulaDir${NC} directory"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to create ${TEAL}$nulaDir${NC} directory"
    exit 1
}

# Move files to appropriate locations
Write-Host "${PINK}┌─[INFO]──${NC} Moving files to system directories..."
Move-Item -Path "$tempDir\nula.exe" -Destination "C:\Program Files\Nula\nula.exe" -Force
Move-Item -Path "$tempDir\nula-zig.exe" -Destination "C:\Program Files\Nula\nula-zig.exe" -Force
Move-Item -Path "$tempDir\nula-go.exe" -Destination "C:\Program Files\Nula\nula-go.exe" -Force
Move-Item -Path "$tempDir\nula-terminal.sh" -Destination "$nulaDir\nula-terminal.sh" -Force
Move-Item -Path "$tempDir\nula.png" -Destination "$nulaDir\nula.png" -Force
Move-Item -Path "$tempDir\nula-lang.desktop" -Destination "$nulaDir\nula-lang.desktop" -Force
$job = Start-Job { Move-Item -Path "$using:tempDir\*" -Destination "C:\Program Files\Nula" -Force }
$i = 0
while ($job.State -eq "Running") {
    Write-Host -NoNewline "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}"
    $i = ($i + 1) % $SPINNER.Length
    Start-Sleep -Milliseconds 100
}
Write-Host "`r" -NoNewline
if ($job.State -eq "Completed") {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} All files moved to their destinations"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Failed to move files"
    exit 1
}

# Add Nula to PATH
Write-Host "${PINK}┌─[INFO]──${NC} Adding Nula to system PATH..."
$envPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($envPath -notlike "*$nulaDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$envPath;$nulaDir", "Machine")
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Added ${TEAL}$nulaDir${NC} to PATH"
} else {
    Write-Host "${YELLOW}└─[INFO]──${NC} ${TEAL}$nulaDir${NC} already in PATH"
}

# Clean up
Write-Host "${PINK}┌─[INFO]──${NC} Cleaning up temporary files..."
Remove-Item -Path $tempDir -Recurse -Force
$job = Start-Job { Remove-Item -Path $using:tempDir -Recurse -Force }
$i = 0
while ($job.State -eq "Running") {
    Write-Host -NoNewline "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}"
    $i = ($i + 1) % $SPINNER.Length
    Start-Sleep -Milliseconds 100
}
Write-Host "`r" -NoNewline
Write-Host "${GREEN}└─[SUCCESS]──${NC} Temporary files removed"

# Create desktop shortcut
Write-Host "${PINK}┌─[INFO]──${NC} Creating desktop shortcut..."
$WShell = New-Object -ComObject WScript.Shell
$shortcut = $WShell.CreateShortcut("$env:USERPROFILE\Desktop\Nula.lnk")
$shortcut.TargetPath = "C:\Program Files\Nula\nula.exe"
$shortcut.IconLocation = "$nulaDir\nula.png"
$shortcut.Save()
Write-Host "${GREEN}└─[SUCCESS]──${NC} Desktop shortcut created"

# Final message
Write-Host ""
Write-Host "${BLUE}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${NC}"
Write-Host "${GREEN}          Nula Programming Language Installed!               ${NC}"
Write-Host "${CYAN} Run ${YELLOW}nula${NC} from PowerShell or double-click the desktop shortcut.${NC}"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"

# Wait for user to admire the output
Write-Host "${ORANGE}┌─[INFO]──${NC} This window will close in 10 seconds..."
Start-Sleep -Seconds 10

# Test Nula installation
Write-Host "${PINK}┌─[INFO]──${NC} Testing Nula installation..."
$job = Start-Job { & "C:\Program Files\Nula\nula.exe" --version }
$i = 0
while ($job.State -eq "Running") {
    Write-Host -NoNewline "`r${VIOLET}$($SPINNER[$i]) ${WHITE}Processing...${NC}"
    $i = ($i + 1) % $SPINNER.Length
    Start-Sleep -Milliseconds 100
}
Write-Host "`r" -NoNewline
if ($job.State -eq "Completed") {
    Write-Host "${GREEN}└─[SUCCESS]──${NC} Nula is working perfectly!"
} else {
    Write-Host "${RED}└─[ERROR]──${NC} Nula installation test failed. Please check the setup."
    exit 1
}

Write-Host "${VIOLET}┌─[THANKS]──${NC} Thank you for installing Nula! Happy coding!"
Write-Host "${BLUE}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${NC}"
