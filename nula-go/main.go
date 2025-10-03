package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"github.com/fatih/color"
)

var (
	globalLibDir string
	cacheDir     string
	indexURL     = "https://raw.githubusercontent.com/Nula-Lang/Nula/main/library/library.toml"
)

func init() {
	home, err := os.UserHomeDir()
	if err != nil {
		home = "."
	}
	globalLibDir = filepath.Join(home, ".nula", "library")
	cacheDir = filepath.Join(home, ".nula", "cache")
	if err := os.MkdirAll(globalLibDir, 0755); err != nil {
		color.Red("Warning: Failed to create global lib dir %s: %v", globalLibDir, err)
	}
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		color.Red("Warning: Failed to create cache dir %s: %v", cacheDir, err)
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
			installCmd := flag.NewFlagSet("install", flag.ExitOnError)
			version := installCmd.String("version", "", "Version to install")
			global := installCmd.Bool("global", true, "Install globally")
			installCmd.Parse(os.Args[2:])
			if installCmd.NArg() < 1 {
				color.Red("Usage: nula-go install <dep> [--version <ver>] [--global]")
				return
			}
			dep := installCmd.Arg(0)
			installDep(dep, *version, *global)
		case "resolve":
			resolveAllDeps()
		case "list":
			listDeps()
		case "remove":
			removeCmd := flag.NewFlagSet("remove", flag.ExitOnError)
			global := removeCmd.Bool("global", true, "Remove globally")
			removeCmd.Parse(os.Args[2:])
			if removeCmd.NArg() < 1 {
				color.Red("Usage: nula-go remove <dep> [--global]")
				return
			}
			removeDep(removeCmd.Arg(0), *global)
		case "update":
			updateCmd := flag.NewFlagSet("update", flag.ExitOnError)
			safe := updateCmd.Bool("safe", true, "Safe update (git pull)")
			force := updateCmd.Bool("force", false, "Force update (reclone)")
				global := updateCmd.Bool("global", true, "Update globally")
				updateCmd.Parse(os.Args[2:])
				updateDeps(*safe, *force, *global)
		case "search":
			searchCmd := flag.NewFlagSet("search", flag.ExitOnError)
			searchCmd.Parse(os.Args[2:])
			if searchCmd.NArg() < 1 {
				color.Red("Usage: nula-go search <query>")
				return
			}
			searchDeps(searchCmd.Arg(0))
		case "update-index":
			updateIndex()
		case "help":
			printHelp()
		default:
			color.Red("Unknown command: %s", cmd)
			printHelp()
	}
}

func printHelp() {
	color.Cyan("nula-go: Dependency manager for Nula")
	fmt.Println("Commands:")
	fmt.Println("  install <dep> [--version <ver>] [--global] - Install a dependency")
	fmt.Println("  remove <dep> [--global] - Remove a dependency")
	fmt.Println("  resolve - Resolve all project dependencies")
	fmt.Println("  list - List installed dependencies")
	fmt.Println("  update [--safe] [--force] [--global] - Update all installed dependencies")
	fmt.Println("  search <query> - Search for dependencies")
	fmt.Println("  update-index - Update the cached library index")
}
