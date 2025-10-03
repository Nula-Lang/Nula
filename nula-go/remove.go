package main

import (
	"os"
	"path/filepath"

	"github.com/fatih/color"
)

func removeDep(dep string, global bool) {
	libDir := getLibDir(global)
	installPath := filepath.Join(libDir, dep)
	if _, err := os.Stat(installPath); os.IsNotExist(err) {
		color.Red("%s not installed", dep)
		return
	}
	if err := os.RemoveAll(installPath); err != nil {
		color.Red("Failed to remove %s: %v", dep, err)
		return
	}
	color.Green("Removed %s", dep)
}
