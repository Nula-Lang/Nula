# PowerShell script to install Nula on Windows
$ErrorActionPreference = "Stop"

# ANSI color codes (approximated for PowerShell)
$RED = "`e[31m"
$GREEN = "`e[32m"
$YELLOW = "`e[33m"
$BLUE = "`e[34m"
$CYAN = "`e[36m"
$NC = "`e[0m"

# Spinner function for visual feedback
function Start-Spinner {
    param (
        [scriptblock]$Command,
        [string]$Message
    )
    $spinstr = '⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
    $job = Start-Job -ScriptBlock $Command
    $delay = 0.1
    while ($job.State -eq "Running") {
        foreach ($i in 0..9) {
            Write-Host -NoNewline "`r${CYAN}$($spinstr[$i])${NC} $Message"
            Start-Sleep -Seconds $delay
        }
    }
    Write-Host "`r${GREEN}✓${NC} $Message"
    Receive-Job -Job $job -Wait
    if ($job.State -eq "Failed") {
        Write-Host "${RED}[ERROR] Command failed: $Message${NC}"
        exit 1
    }
    Remove-Job -Job $job
}

Write-Host "${BLUE}[INFO] Checking operating system...${NC}"

# Detect operating system
$DISTRO = "windows"
if ($env:OS -like "*Windows*") {
    $DISTRO = "windows"
}
Write-Host "${GREEN}[INFO] Operating system detected: $DISTRO${NC}"

# Determine if this is an atomic/immutable distribution (not typically applicable on Windows)
$is_atomic = $false
# For demonstration, allow manual override or specific cases
# No atomic distributions on Windows by default, but keeping logic for consistency
Write-Host "${YELLOW}[INFO] Windows is not an atomic distribution. Binaries will be placed in standard paths unless overridden.${NC}"

# Function to install dependencies
function Install-Dependency {
    param (
        [string]$dep
    )
    $pkg = $dep
    switch ($dep) {
        "go" { $pkg = "golang" }
        "zig" { $pkg = "zig" }
        "gcc" { $pkg = "mingw" } # Use MinGW for gcc on Windows
        "git" { $pkg = "git" }
        "curl" { $pkg = "curl" }
    }

    if (Get-Command $dep -ErrorAction SilentlyContinue) {
        Write-Host "${GREEN}[INFO] $dep is already installed.${NC}"
        return
    }

    Write-Host "${YELLOW}[INFO] $dep not found, installing...${NC}"
    if (Get-Command "choco" -ErrorAction SilentlyContinue) {
        # Use Chocolatey if available
        Start-Spinner -Command { choco install $pkg -y } -Message "Installing $dep via Chocolatey..."
    }
    elseif (Get-Command "winget" -ErrorAction SilentlyContinue) {
        # Use winget if available
        Start-Spinner -Command { winget install --id $pkg --silent } -Message "Installing $dep via winget..."
    }
    else {
        Write-Host "${RED}[ERROR] No package manager (Chocolatey or winget) found. Please install $pkg manually.${NC}"
        exit 1
    }
}

# Required dependencies
$deps = @("git", "curl", "go", "zig", "gcc")

# Check and install dependencies
Write-Host "${BLUE}[INFO] Checking and installing dependencies...${NC}"
foreach ($dep in $deps) {
    Install-Dependency -dep $dep
}

# Check Rust
if (-not (Get-Command "rustc" -ErrorAction SilentlyContinue)) {
    Write-Host "${YELLOW}[INFO] Rust not found, installing via rustup...${NC}"
    Start-Spinner -Command {
        Invoke-WebRequest -Uri "https://sh.rustup.rs" -OutFile "$env:TEMP\rustup-init.sh"
        & "$env:TEMP\rustup-init.sh" -y --default-toolchain stable
        Remove-Item "$env:TEMP\rustup-init.sh"
    } -Message "Installing Rust..."
    # Update environment to include Cargo
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}
else {
    Write-Host "${GREEN}[INFO] Rust is already installed.${NC}"
}

# Clone repository
Write-Host "${BLUE}[RUN] Cloning the repository...${NC}"
Start-Spinner -Command { git clone https://github.com/Nula-Lang/Nula.git $env:TEMP\Nula } -Message "Cloning Nula repository..."
Set-Location "$env:TEMP\Nula"

# Create user-local directories
$null = New-Item -Path "$env:USERPROFILE\.nula\lib" -ItemType Directory -Force

# Build Nula (Go)
Write-Host "${BLUE}[RUN] Building nula (Go)...${NC}"
Set-Location "$env:TEMP\Nula\nula\go"
if (-not (Test-Path "go.mod")) {
    Start-Spinner -Command { go mod init example.com/m/v2 } -Message "Initializing Go module..."
}
Start-Spinner -Command { go mod tidy } -Message "Tidying Go modules..."
Start-Spinner -Command { go build } -Message "Building nula (Go)..."
Rename-Item -Path "m.exe" -NewName "nula-go.exe"
$null = chmod +x "nula-go.exe" # chmod not needed on Windows, included for consistency
Move-Item -Path "nula-go.exe" -Destination "$env:USERPROFILE\.nula\lib\"

# Build Nula (Zig)
Set-Location "$env:TEMP\Nula\nula\zig"
Write-Host "${BLUE}[RUN] Building nula (Zig)...${NC}"
Start-Spinner -Command { zig build-exe main.zig -O ReleaseFast } -Message "Building nula (Zig)..."
Rename-Item -Path "main.exe" -NewName "nula-zig.exe"
$null = chmod +x "nula-zig.exe" # chmod not needed on Windows
Move-Item -Path "nula-zig.exe" -Destination "$env:USERPROFILE\.nula\lib\"

# Build Nula (Rust)
Set-Location "$env:TEMP\Nula\nula"
Write-Host "${BLUE}[RUN] Building nula (Rust)...${NC}"
Start-Spinner -Command { cargo build --release } -Message "Building nula (Rust)..."
Set-Location "target\release"
$null = chmod +x "nula.exe" # chmod not needed on Windows

# Install the Rust binary based on distribution type
if ($is_atomic) {
    $null = New-Item -Path "$env:USERPROFILE\.local\bin" -ItemType Directory -Force
    Move-Item -Path "nula.exe" -Destination "$env:USERPROFILE\.local\bin\"
    Write-Host "${GREEN}[INFO] Installed nula binary to $env:USERPROFILE\.local\bin\ for atomic distribution.${NC}"
    Write-Host "${CYAN}[PLEASE] Ensure $env:USERPROFILE\.local\bin\ is in your PATH. Run 'nula' from there.${NC}"
}
else {
    # On Windows, use a system-wide directory like Program Files or AppData
    $installPath = "$env:ProgramFiles\Nula"
    $null = New-Item -Path $installPath -ItemType Directory -Force
    Move-Item -Path "nula.exe" -Destination "$installPath\"
    # Add to PATH if not already present
    if (-not ($env:PATH -like "*$installPath*")) {
        $env:PATH += ";$installPath"
        [Environment]::SetEnvironmentVariable("Path", $env:PATH, [System.EnvironmentVariableTarget]::User)
    }
    Write-Host "${GREEN}[INFO] Installed nula binary to $installPath\.${NC}"
}

Write-Host "${GREEN}[INFO] The operation has been completed successfully!${NC}"
Write-Host "${CYAN}[PLEASE] Run the nula command or launch the application from the nula program menu.${NC}"
