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
	var home string
	if runtime.GOOS == "windows" {
		home = os.Getenv("APPDATA")
	} else {
		home = os.Getenv("HOME")
	}
	if home == "" {
		home = "."
	}
	libDir = filepath.Join(home, ".nula", "lib")
	if err := os.MkdirAll(libDir, 0755); err != nil {
		fmt.Printf("Warning: Failed to create lib dir %s: %v\n", libDir, err)
	}
}

func main() {
	if len(os.Args) < 2 {
		printHelp()
		return
	}

	cmd := os.Args[1]
	switch cmd {
		case "install":
			if len(os.Args) < 3 {
				fmt.Println("Usage: nula-go install <dep> [--version <ver>]")
				return
			}
			dep := os.Args[2]
			version := ""
			if len(os.Args) > 4 && os.Args[3] == "--version" {
				version = os.Args[4]
			}
			installDep(dep, version)
		case "resolve":
			resolveAllDeps()
		case "list":
			listDeps()
		case "remove":
			if len(os.Args) < 3 {
				fmt.Println("Usage: nula-go remove <dep>")
				return
			}
			removeDep(os.Args[2])
		case "help":
			printHelp()
		default:
			fmt.Println("Unknown command:", cmd)
			printHelp()
	}
}

func printHelp() {
	fmt.Println("nula-go: Dependency manager for Nula")
	fmt.Println("Commands:")
	fmt.Println("  install <dep> [--version <ver>] - Install a dependency")
	fmt.Println("  remove <dep> - Remove a dependency")
	fmt.Println("  resolve - Resolve all project dependencies")
	fmt.Println("  list - List installed dependencies")
}

func installDep(dep, version string) {
	indexPath := filepath.Join(os.TempDir(), "library.nula")
	curlCmd := exec.Command("curl", "-s", "-o", indexPath, "https://raw.githubusercontent.com/Nula-Lang/Nula/main/nula/library.nula")
	if err := curlCmd.Run(); err != nil {
		fmt.Printf("Failed to fetch library index: %v\n", err)
		return
	}
	defer os.Remove(indexPath)

	file, err := os.Open(indexPath)
	if err != nil {
		fmt.Printf("Failed to open index: %v\n", err)
		return
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.SplitN(line, "_>", 2)
		if len(parts) != 2 {
			continue
		}
		left := strings.TrimSpace(parts[0])
		url := strings.TrimSpace(parts[1])
		leftParts := strings.SplitN(left, ":", 2)
		if len(leftParts) != 2 || strings.TrimSpace(leftParts[0]) != dep {
			continue
		}
		typ := strings.TrimSpace(leftParts[1])
		installPath := filepath.Join(libDir, dep)
		if version != "" {
			installPath = filepath.Join(installPath, version)
		}
		if _, err := os.Stat(installPath); err == nil {
			fmt.Printf("%s %s already installed\n", dep, version)
			return
		}
		if err := os.MkdirAll(installPath, 0755); err != nil {
			fmt.Printf("Failed to create dir: %v\n", err)
			return
		}
		switch typ {
			case "git":
				cloneCmd := exec.Command("git", "clone", "--depth=1")
				if version != "" {
					cloneCmd.Args = append(cloneCmd.Args, "-b", version)
				}
				cloneCmd.Args = append(cloneCmd.Args, url, installPath)
				cloneCmd.Stdout = os.Stdout
				cloneCmd.Stderr = os.Stderr
				if err := cloneCmd.Run(); err != nil {
					fmt.Printf("Failed to clone: %v\n", err)
					return
				}
				fmt.Printf("Installed git dep %s %s\n", dep, version)
			case "bin":
				filename := filepath.Base(url)
				destPath := filepath.Join(installPath, filename)
				curlBinCmd := exec.Command("curl", "-L", "-o", destPath, url)
				curlBinCmd.Stdout = os.Stdout
				curlBinCmd.Stderr = os.Stderr
				if err := curlBinCmd.Run(); err != nil {
					fmt.Printf("Failed to download bin: %v\n", err)
					return
				}
				if runtime.GOOS != "windows" {
					if err := os.Chmod(destPath, 0755); err != nil {
						fmt.Printf("Failed to make executable: %v\n", err)
					}
				}
				fmt.Printf("Installed bin dep %s %s\n", dep, version)
			default:
				fmt.Printf("Unknown type %s\n", typ)
				return
		}
		return
	}
	fmt.Printf("Dep %s not found\n", dep)
}

func removeDep(dep string) {
	installPath := filepath.Join(libDir, dep)
	if _, err := os.Stat(installPath); os.IsNotExist(err) {
		fmt.Printf("%s not installed\n", dep)
		return
	}
	if err := os.RemoveAll(installPath); err != nil {
		fmt.Printf("Failed to remove %s: %v\n", dep, err)
		return
	}
	fmt.Printf("Removed %s\n", dep)
}

func resolveAllDeps() {
	config, err := os.ReadFile("nula.toml")
	if err != nil {
		fmt.Printf("Failed to read nula.toml: %v\n", err)
		return
	}
	inDeps := false
	deps := make(map[string]string)
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
			parts := strings.SplitN(trimmed, "=", 2)
			dep := strings.TrimSpace(parts[0])
			ver := strings.TrimSpace(strings.Trim(parts[1], "\""))
			if dep != "" {
				deps[dep] = ver
			}
		}
	}

	for dep, ver := range deps {
		installDep(dep, ver)
	}

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
					if dep != "" && deps[dep] == "" {
						installDep(dep, "")
					}
				}
			}
		}
		return nil
	})
	if err != nil {
		fmt.Printf("Failed to scan: %v\n", err)
	}
	fmt.Println("All deps resolved")
}

func listDeps() {
	entries, err := os.ReadDir(libDir)
	if err != nil {
		fmt.Printf("Failed to list: %v\n", err)
		return
	}
	fmt.Println("Installed deps:")
	for _, entry := range entries {
		if entry.IsDir() {
			subEntries, _ := os.ReadDir(filepath.Join(libDir, entry.Name()))
			if len(subEntries) > 0 {
				for _, sub := range subEntries {
					fmt.Printf("%s (%s)\n", entry.Name(), sub.Name())
				}
			} else {
				fmt.Println(entry.Name())
			}
		}
	}
}
