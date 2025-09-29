package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

var libDir string

func init() {
	// Cross-platform lib dir
	var home string
	if runtime.GOOS == "windows" {
		home = os.Getenv("APPDATA")
	} else {
		home = os.Getenv("HOME")
	}
	if home == "" {
		home = "."
	}
	libDir = filepath.Join(home, ".nula-lib")
	if err := os.MkdirAll(libDir, 0755); err != nil {
		fmt.Printf("Warning: Failed to create lib dir %s: %v\n", libDir, err)
	}
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("nula-go: Missing command")
		return
	}

	cmd := os.Args[1]
	switch cmd {
	case "install":
		if len(os.Args) < 3 {
			fmt.Println("Usage: nula-go install <dep>")
			return
		}
		dep := os.Args[2]
		installDep(dep)
	case "resolve":
		resolveAllDeps()
	case "list":
		listDeps()
	default:
		fmt.Println("Unknown command:", cmd)
	}
}

func installDep(dep string) {
	// Fetch library.nula from GitHub (use curl)
	curlCmd := exec.Command("curl", "-s", "https://raw.githubusercontent.com/Nula-Lang/Nula/main/nula/library.nula", "-o", filepath.Join(os.TempDir(), "library.nula"))
	curlCmd.Stderr = os.Stderr
	if err := curlCmd.Run(); err != nil {
		fmt.Printf("Failed to fetch library list: %v\n", err)
		return
	}

	// Read and parse
	filePath := filepath.Join(os.TempDir(), "library.nula")
	file, err := os.Open(filePath)
	if err != nil {
		fmt.Printf("Failed to open library: %v\n", err)
		return
	}
	defer file.Close()
	defer os.Remove(filePath)

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.SplitN(line, " -> ", 2)
		if len(parts) == 2 && strings.TrimSpace(parts[0]) == dep {
			url := strings.TrimSpace(parts[1])
			clonePath := filepath.Join(libDir, dep)
			if _, err := os.Stat(clonePath); err == nil {
				fmt.Printf("%s already installed at %s\n", dep, clonePath)
				return
			}
			cloneCmd := exec.Command("git", "clone", "--depth=1", url, clonePath)
			cloneCmd.Stdout = os.Stdout
			cloneCmd.Stderr = os.Stderr
			if err := cloneCmd.Run(); err != nil {
				fmt.Printf("Failed to clone %s: %v\n", dep, err)
				return
			}
			fmt.Printf("Installed %s from %s to %s\n", dep, url, clonePath)
			return
		}
	}
	if err := scanner.Err(); err != nil {
		fmt.Printf("Error reading library: %v\n", err)
	}
	fmt.Printf("Dependency %s not found in library list\n", dep)
}

func resolveAllDeps() {
	// Parse nula.toml for [dependencies]
	config, err := os.ReadFile("nula.toml")
	if err != nil {
		fmt.Printf("Failed to read nula.toml: %v\n", err)
		return
	}
	inDeps := false
	lines := strings.Split(string(config), "\n")
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "[dependencies]" {
			inDeps = true
			continue
		}
		if inDeps && (strings.HasPrefix(trimmed, "[") || trimmed == "") {
			inDeps = false
			continue
		}
		if inDeps && strings.Contains(trimmed, "=") {
			dep := strings.SplitN(trimmed, "=", 2)[0]
			dep = strings.TrimSpace(dep)
			if dep != "" {
				installDep(dep)
			}
		}
	}

	// Also scan .nula files for <dep>
	err = filepath.Walk(".", func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(path, ".nula") {
			content, err := os.ReadFile(path)
			if err != nil {
				fmt.Printf("Failed to read %s: %v\n", path, err)
				return nil
			}
			lines := strings.Split(string(content), "\n")
			for _, line := range lines {
				trimmed := strings.TrimSpace(line)
				if strings.HasPrefix(trimmed, "<") && strings.HasSuffix(trimmed, ">") {
					dep := strings.Trim(trimmed, "<>")
					dep = strings.TrimSpace(dep)
					if dep != "" {
						installDep(dep)
					}
				}
			}
		}
		return nil
	})
	if err != nil {
		fmt.Printf("Failed to scan project: %v\n", err)
	}
	fmt.Println("All dependencies resolved")
}

func listDeps() {
	entries, err := os.ReadDir(libDir)
	if err != nil {
		fmt.Printf("Failed to list deps: %v\n", err)
		return
	}
	fmt.Println("Installed dependencies:")
	for _, entry := range entries {
		if entry.IsDir() {
			fmt.Println(entry.Name())
		}
	}
}
