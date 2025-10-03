package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/fatih/color"
)

func listDeps() {
	entries, err := os.ReadDir(globalLibDir)
	if err != nil {
		color.Red("Failed to read global lib directory %s: %v", globalLibDir, err)
		return
	}
	localDir := getLibDir(false)
	localEntries, _ := os.ReadDir(localDir)

	allDeps := append(entries, localEntries...)
	if len(allDeps) == 0 {
		fmt.Println("No dependencies installed")
		return
	}
	color.Cyan("Installed dependencies:")
	for _, entry := range allDeps {
		if entry.IsDir() {
			dep := entry.Name()
			depPath := filepath.Join(globalLibDir, dep)
			if _, err := os.Stat(depPath); os.IsNotExist(err) {
				depPath = filepath.Join(localDir, dep)
			}
			metaPath := filepath.Join(depPath, "metadata.json")
			var meta Metadata
			if data, err := os.ReadFile(metaPath); err == nil {
				json.Unmarshal(data, &meta)
				fmt.Printf("%s: %s (%s, %s)\n", meta.Dep, meta.Version, meta.Type, meta.URL)
			} else {
				fmt.Printf("%s (no metadata)\n", dep)
			}
		}
	}
}
