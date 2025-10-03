package main

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/fatih/color"
)

type Lockfile struct {
	Dependencies map[string]struct {
		Version string `toml:"version"`
		Type    string `toml:"type"`
		URL     string `toml:"url"`
		Source  string `toml:"source"`
	} `toml:"dependencies"`
}

func resolveAllDeps() {
	lockPath := "nula.lock"
	var lock Lockfile
	if _, err := toml.DecodeFile(lockPath, &lock); err == nil {
		for dep, info := range lock.Dependencies {
			installDep(dep, info.Version, false)
		}
		color.Green("Resolved from lockfile")
		return
	}

	config, err := os.ReadFile("nula.toml")
	if err != nil {
		color.Red("Failed to read nula.toml: %v", err)
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
		installDep(dep, ver, false)
	}

	err = filepath.Walk(".", func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() && strings.HasSuffix(path, ".nula") {
			content, err := os.ReadFile(path)
			if err != nil {
				color.Red("Failed to read %s: %v", path, err)
				return nil
			}
			lines := strings.Split(string(content), "\n")
			for _, line := range lines {
				trimmed := strings.TrimSpace(line)
				if strings.HasPrefix(trimmed, "<") && strings.HasSuffix(trimmed, ">") {
					dep := strings.Trim(trimmed, "<>")
					dep = strings.TrimSpace(dep)
					if dep != "" && deps[dep] == "" {
						installDep(dep, "", false)
					}
				}
			}
		}
		return nil
	})
	if err != nil {
		color.Red("Failed to scan: %v", err)
	}
	color.Green("All deps resolved")
}

func updateLockfile(dep, version, typ, url, libDir string) {
	lockPath := filepath.Join(libDir, "nula.lock")
	if libDir == globalLibDir {
		lockPath = "nula.lock"
	}
	var lock Lockfile
	if data, err := os.ReadFile(lockPath); err == nil {
		toml.Decode(string(data), &lock)
	}
	if lock.Dependencies == nil {
		lock.Dependencies = make(map[string]struct {
			Version string `toml:"version"`
			Type    string `toml:"type"`
			URL     string `toml:"url"`
			Source  string `toml:"source"`
		})
	}
	lock.Dependencies[dep] = struct {
		Version string `toml:"version"`
		Type    string `toml:"type"`
		URL     string `toml:"url"`
		Source  string `toml:"source"`
	}{Version: version, Type: typ, URL: url, Source: libDir}
	out, err := os.Create(lockPath)
	if err != nil {
		color.Red("Failed to create lockfile: %v", err)
		return
	}
	defer out.Close()
	if err := toml.NewEncoder(out).Encode(lock); err != nil {
		color.Red("Failed to encode lockfile: %v", err)
	}
}
