package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/charmbracelet/bubbletea"
	"github.com/fatih/color"
)

func updateDeps(safe, force, global bool) {
	libDir := getLibDir(global)
	entries, err := os.ReadDir(libDir)
	if err != nil {
		color.Red("Failed to read lib directory %s: %v", libDir, err)
		return
	}
	if len(entries) == 0 {
		fmt.Println("No dependencies to update")
		return
	}

	p := tea.NewProgram(spinnerModel{})
	go func() {
		for _, entry := range entries {
			if entry.IsDir() {
				dep := entry.Name()
				installPath := filepath.Join(libDir, dep)
				metaPath := filepath.Join(installPath, "metadata.json")
				var meta Metadata
				if data, err := os.ReadFile(metaPath); err == nil {
					json.Unmarshal(data, &meta)
				}
				if force {
					os.RemoveAll(installPath)
					installDep(dep, meta.Version, global)
				} else if safe && meta.Type == "git" {
					cmd := exec.Command("git", "-C", installPath, "pull")
					cmd.Stdout = os.Stdout
					cmd.Stderr = os.Stderr
					if err := cmd.Run(); err != nil {
						color.Yellow("Failed to pull %s: %v", dep, err)
					}
					cmd = exec.Command("git", "-C", installPath, "fetch", "--tags")
					cmd.Run()
				} else {
					os.RemoveAll(installPath)
					installDep(dep, meta.Version, global)
				}
			}
		}
		p.Send(spinnerDone{})
	}()
	if _, err := p.Run(); err != nil {
		color.Red("Spinner error: %v", err)
	}
	color.Green("All dependencies updated")
}
