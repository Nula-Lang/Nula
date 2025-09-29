package deps

import (
    "fmt"
    "os"
    "os/exec"
    "path/filepath"
    "runtime"
    "strings"

    "github.com/fatih/color"
)

// InstallDependency downloads and installs a dependency from library.nula.
func InstallDependency(dep string) {
    libDir := getLibDir()
    if err := os.MkdirAll(libDir, 0755); err != nil {
        errorMsg(fmt.Sprintf("Failed to create library directory %s: %v", libDir, err))
        return
    }

    // Fetch library.nula from GitHub
    libraryURL := "https://raw.githubusercontent.com/Nula-Lang/Nula/main/library.nula"
    cmd := exec.Command("curl", "-s", libraryURL)
    output, err := cmd.Output()
    if err != nil {
        errorMsg(fmt.Sprintf("Failed to fetch %s: %v", libraryURL, err))
        return
    }

    // Parse library.nula
    repoURL := ""
    lines := strings.Split(string(output), "\n")
    for _, line := range lines {
        if strings.HasPrefix(line, dep+" -> ") {
            repoURL = strings.TrimSpace(strings.TrimPrefix(line, dep+" -> "))
            break
        }
    }

    if repoURL == "" {
        errorMsg(fmt.Sprintf("Dependency %s not found in library.nula", dep))
        return
    }

    // Clone the repository
    clonePath := filepath.Join(libDir, dep)
    cmd = exec.Command("git", "clone", repoURL, clonePath)
    cmd.Stdout = os.Stdout
    cmd.Stderr = os.Stderr
    if err := cmd.Run(); err != nil {
        errorMsg(fmt.Sprintf("Failed to clone %s: %v", repoURL, err))
        return
    }

    color.Green("Successfully installed %s to %s", dep, clonePath)
}

// getLibDir returns the platform-specific library directory.
func getLibDir() string {
    if runtime.GOOS == "windows" {
        systemRoot := os.Getenv("SystemRoot")
        if systemRoot == "" {
            systemRoot = `C:\Windows`
        }
        return filepath.Join(systemRoot, "System32", ".nula-lib")
    }
    return "/usr/lib/.nula-lib"
}

// errorMsg prints an error message in red and exits.
func errorMsg(msg string) {
    color.Red("ERROR: %s", msg)
    os.Exit(1)
}
