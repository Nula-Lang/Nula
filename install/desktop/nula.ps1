# open-nula.ps1
# Uruchamia "nula help" w domyślnym terminalu Windows:
# - najpierw próbuje Windows Terminal (wt.exe)
# - następnie PowerShell (nowe okno)
# - w ostateczności CMD
# Pozostawia okno otwarte po wykonaniu komendy.

param(
    [string]$Cmd = "nula help"
)

# Funkcja do uruchomienia w Windows Terminal (wt)
function Start-WithWindowsTerminal {
    param([string]$command)
    $wt = (Get-Command wt.exe -ErrorAction SilentlyContinue)
    if ($null -ne $wt) {
        # utwórz nową kartę/okno z PowerShell i wykonaj komendę, zostaw okno otwarte
        # używamy powershell -NoExit -Command "..."
        & wt new-tab powershell -NoExit -Command $command
        return $true
    }
    return $false
}

# Funkcja do uruchomienia w nowym oknie PowerShell (classic)
function Start-WithPowerShell {
    param([string]$command)
    $pwshCmd = "$env:WINDIR\System32\WindowsPowerShell\v1.0\powershell.exe"
    if (Test-Path $pwshCmd) {
        Start-Process -FilePath $pwshCmd -ArgumentList "-NoExit","-Command",$command -WindowStyle Normal
        return $true
    }
    return $false
}

# Funkcja do uruchomienia w nowym oknie cmd.exe
function Start-WithCmd {
    param([string]$command)
    $cmdExe = "$env:WINDIR\System32\cmd.exe"
    if (Test-Path $cmdExe) {
        # /k zostawia okno otwarte
        Start-Process -FilePath $cmdExe -ArgumentList "/k", $command -WindowStyle Normal
        return $true
    }
    return $false
}

# Spróbuj po kolei
if (Start-WithWindowsTerminal -command $Cmd) {
    exit 0
}
elseif (Start-WithPowerShell -command $Cmd) {
    exit 0
}
elseif (Start-WithCmd -command $Cmd) {
    exit 0
}
else {
    # fallback: uruchom w bieżącym oknie PowerShell
    Write-Host "Nie znaleziono GUI terminala. Uruchamiam w bieżącym oknie..."
    Invoke-Expression $Cmd
    exit $LASTEXITCODE
}
