# create-shortcut.ps1
$desktop = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktop "Open Nula Help.lnk"

# target: możemy wskazać .bat lub powershell z parametrem
$target = "C:\Tools\open-nula.bat"
$workingDir = Split-Path $target

$WshShell = New-Object -ComObject WScript.Shell
$shortcut = $WshShell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $target
$shortcut.WorkingDirectory = $workingDir
$shortcut.WindowStyle = 1
$shortcut.IconLocation = "shell32.dll, 221"  # można zmienić na ścieżkę do własnej ikony
$shortcut.Save()

Write-Host "Skrót utworzony: $shortcutPath"
