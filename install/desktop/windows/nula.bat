@echo off
REM Ścieżkę ustaw na lokalizację nula.ps1
SET PSFILE=C:\Tools\nula.ps1

REM Jeśli potrzebujesz przekazać inną komendę, możesz dodać ją jako parametr do .bat
REM Przykład: open-nula.bat "nula help"
IF "%~1"=="" (
  powershell -NoProfile -ExecutionPolicy Bypass -File "%PSFILE%"
) ELSE (
  powershell -NoProfile -ExecutionPolicy Bypass -File "%PSFILE%" -Cmd "%~1"
)
