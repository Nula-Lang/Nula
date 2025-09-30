# PowerShell script to install Nula on Windows

# ANSI color codes for PowerShell (approximated)
$RED = "`e[31m"
$GREEN = "`e[32m"
$YELLOW = "`e[33m"
$BLUE = "`e[34m"
$CYAN = "`e[36m"
$NC = "`e[0m"

# Function to display a spinner
function Show-Spinner {
    param (
        [scriptblock]$Command,
        [string]$Message
    )
    $spinstr = @('⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏')
    $job = Start-Job -ScriptBlock $Command
    while ($job.State -eq 'Running') {
        foreach ($char in $spinstr) {
            Write-Host -NoNewline "`r${CYAN}${char}${NC} $Message"
            Start-Sleep -Milliseconds 100
        }
    }
    $result = Receive-Job -Job $job
    Write-Host "`r${GREEN}✓${NC} $Message"
    return $result
}

Write-Host "${BLUE}[INFO] Checking system...${NC}"

# Ensure running with elevated privileges
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "${RED}[ERROR] This script requires administrative privileges. Please run as Administrator.${NC}"
    exit 1
}

# Function to install dependencies
function Install-Dependency {
    param (
        [string]$Dep
    )
    $pkg = $Dep
    switch ($Dep) {
        "go" { $pkg = "golang" }
        "zig" { $pkg = "zig" }
        "gcc" { $pkg = "mingw" } # Use MinGW for gcc on Windows
        "git" { $pkg = "git" }
        "curl" { $pkg = "curl" }
    }

    if (-not (Get-Command $Dep -ErrorAction SilentlyContinue)) {
        Write-Host "${YELLOW}[INFO] $Dep not found, installing...${NC}"
        if ($Dep -eq "gcc") {
            # Install MinGW via winget
            Show-Spinner -Command { winget install --id msys2.msys2 --source winget --silent } -Message "Installing MinGW (gcc)..."
            # Add MinGW to PATH
            $env:Path += ";C:\msys64\mingw64\bin"
            [Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::Machine)
        } else {
            Show-Spinner -Command { winget install --id $pkg --source winget --silent } -Message "Installing $Dep..."
        }
    } else {
        Write-Host "${GREEN}[INFO] $Dep is already installed.${NC}"
    }
}

# Required dependencies
$deps = @("git", "curl", "go", "zig", "gcc")

# Check and install dependencies
Write-Host "${BLUE}[INFO] Checking and installing dependencies...${NC}"
foreach ($dep in $deps) {
    Install-Dependency -Dep $dep
}

# Check Rust
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "${YELLOW}[INFO] Rust not found, installing via rustup...${NC}"
    Show-Spinner -Command { Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"; & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable } -Message "Installing Rust..."
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
    [Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::Machine)
} else {
    Write-Host "${GREEN}[INFO] Rust is already installed.${NC}"
}

# Clone repository
Write-Host "${BLUE}[RUN] Cloning the repository...${NC}"
Show-Spinner -Command { git clone https://github.com/Nula-Lang/Nula.git $env:TEMP\Nula } -Message "Cloning Nula repository..."
Set-Location $env:TEMP\Nula

# Build Nula (Go)
Write-Host "${BLUE}[RUN] Building nula (Go)...${NC}"
Set-Location nula\go
if (-not (Test-Path go.mod)) {
    Show-Spinner -Command { go mod init example.com/m/v2 } -Message "Initializing Go module..."
}
Show-Spinner -Command { go mod tidy } -Message "Tidying Go modules..."
Show-Spinner -Command { go build -o nula-go.exe } -Message "Building nula (Go)..."
Move-Item -Force nula-go.exe C:\Windows\System32\

# Build Nula (Zig)
Write-Host "${BLUE}[RUN] Building nula (Zig)...${NC}"
Set-Location ..\zig
Show-Spinner -Command { zig build-exe main.zig -O ReleaseFast } -Message "Building nula (Zig)..."
Rename-Item main.exe nula-zig.exe
Move-Item -Force nula-zig.exe C:\Windows\System32\

# Build Nula (Rust)
Write-Host "${BLUE}[RUN] Building nula (Rust)...${NC}"
Set-Location ..
Show-Spinner -Command { cargo build --release } -Message "Building nula (Rust)..."
Move-Item -Force target\release\nula.exe C:\Windows\System32\

# Clean up
Set-Location $env:TEMP
Remove-Item -Recurse -Force Nula

Write-Host "${GREEN}[INFO] The operation has been completed successfully!${NC}"
Write-Host "${CYAN}[PLEASE] Run the nula, nula-go, or nula-zig command from the command prompt.${NC}"
