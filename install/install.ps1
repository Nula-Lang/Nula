# Nula Installer for Windows
Write-Host "[DOWNLOAD]"

# Ustawienie folderu tymczasowego
$temp = $env:TEMP

# Pobieranie plików
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/releases/download/v0.1/nula-zig" -OutFile "$temp\nula-zig"
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/releases/download/v0.1/nula-go" -OutFile "$temp\nula-go"
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/releases/download/v0.1/nula" -OutFile "$temp\nula"
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/nula.png" -OutFile "$temp\nula.png"
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/windows/nula.bat" -OutFile "$temp\nula.bat"
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/windows/nula.Ink" -OutFile "$temp\nula.Ink"
Invoke-WebRequest -Uri "https://github.com/Nula-Lang/Nula/raw/main/install/desktop/windows/nula.ps1" -OutFile "$temp\nula.ps1"

Write-Host "[INFO] Moving files..."

# Tworzenie folderu dla plików Nula
$installFolder = "$env:ProgramFiles\Nula"
if (-not (Test-Path $installFolder)) {
    New-Item -ItemType Directory -Path $installFolder
}

# Kopiowanie plików
Copy-Item "$temp\nula" -Destination $installFolder -Force
Copy-Item "$temp\nula-zig" -Destination $installFolder -Force
Copy-Item "$temp\nula-go" -Destination $installFolder -Force
Copy-Item "$temp\nula.bat" -Destination $installFolder -Force
Copy-Item "$temp\nula.ps1" -Destination $installFolder -Force
Copy-Item "$temp\nula.Ink" -Destination "$env:Public\Desktop" -Force
Copy-Item "$temp\nula.png" -Destination "$env:ProgramFiles\Nula" -Force

Write-Host "[INFO] The Nula programming language has been installed."
Write-Host "[INFO] Run the nula.bat or nula.ps1 to start the Nula environment."

# Czekanie 10 sekund
Start-Sleep -Seconds 10

# Uruchomienie terminala PowerShell
powershell
