package main

import (
	"archive/tar"
	"archive/zip"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/Masterminds/semver/v3"
	"github.com/charmbracelet/bubbletea"
	"github.com/fatih/color"
)

type LibraryIndex map[string]struct {
	Type string `toml:"type"`
	URL  string `toml:"url"`
}

type Metadata struct {
	Dep     string `json:"dep"`
	Version string `json:"version"`
	Type    string `json:"type"`
	URL     string `json:"url"`
}

func getLibDir(global bool) string {
	if global {
		return globalLibDir
	}
	localDir := filepath.Join(".", "nula_libs")
	if err := os.MkdirAll(localDir, 0755); err != nil {
		color.Red("Failed to create local lib dir: %v", err)
	}
	return localDir
}

func getIndexPath() string {
	return filepath.Join(cacheDir, "library.toml")
}

func fetchIndexIfNeeded() {
	indexPath := getIndexPath()
	if _, err := os.Stat(indexPath); os.IsNotExist(err) {
		updateIndex()
	}
}

func updateIndex() {
	color.Yellow("Updating library index...")
	p := tea.NewProgram(spinnerModel{})
	go func() {
		resp, err := http.Get(indexURL)
		if err != nil {
			color.Red("Failed to fetch library index: %v", err)
			return
		}
		defer resp.Body.Close()
		out, err := os.Create(getIndexPath())
		if err != nil {
			color.Red("Failed to save index: %v", err)
			return
		}
		defer out.Close()
		if _, err := io.Copy(out, resp.Body); err != nil {
			color.Red("Failed to copy index: %v", err)
			return
		}
		p.Send(spinnerDone{})
	}()
	if _, err := p.Run(); err != nil {
		color.Red("Spinner error: %v", err)
	}
	color.Green("Index updated")
}

func loadIndex() LibraryIndex {
	fetchIndexIfNeeded()
	indexPath := getIndexPath()
	var index LibraryIndex
	if _, err := toml.DecodeFile(indexPath, &index); err != nil {
		color.Red("Failed to parse library.toml: %v", err)
		os.Exit(1)
	}
	return index
}

func installDep(dep, version string, global bool) {
	index := loadIndex()
	info, ok := index[dep]
	if !ok {
		color.Red("Dependency %s not found", dep)
		return
	}

	libDir := getLibDir(global)
	installPath := filepath.Join(libDir, dep)
	if version != "" {
		_, err := semver.NewConstraint(version)
		if err != nil {
			color.Red("Invalid version constraint: %v", err)
			return
		}
		installPath = filepath.Join(installPath, version)
	}
	if _, err := os.Stat(installPath); err == nil {
		color.Yellow("%s %s already installed", dep, version)
		return
	}
	if err := os.MkdirAll(installPath, 0755); err != nil {
		color.Red("Failed to create dir: %v", err)
		return
	}

	p := tea.NewProgram(spinnerModel{})
	go func() {
		switch info.Type {
			case "git":
				installGitDep(info.URL, installPath, version)
			case "bin":
				installBinDep(info.URL, installPath)
			case "tar":
				installTarDep(info.URL, installPath)
			case "zip":
				installZipDep(info.URL, installPath)
			case "http":
				installHttpDep(info.URL, installPath)
			default:
				color.Red("Unknown type %s", info.Type)
				return
		}
		meta := Metadata{Dep: dep, Version: version, Type: info.Type, URL: info.URL}
		metaPath := filepath.Join(installPath, "metadata.json")
		metaFile, err := os.Create(metaPath)
		if err != nil {
			color.Red("Failed to create metadata: %v", err)
			return
		}
		defer metaFile.Close()
		if err := json.NewEncoder(metaFile).Encode(meta); err != nil {
			color.Red("Failed to encode metadata: %v", err)
			return
		}
		if !global {
			addToNulaToml(dep, version)
		}
		updateLockfile(dep, version, info.Type, info.URL, libDir)
		p.Send(spinnerDone{})
	}()
	if _, err := p.Run(); err != nil {
		color.Red("Spinner error: %v", err)
	}
	color.Green("Installed %s %s (%s, %s)", dep, version, info.Type, info.URL)
}

func installGitDep(url, path, version string) {
	cmd := exec.Command("git", "clone", "--depth=1")
	if version != "" {
		cmd.Args = append(cmd.Args, "-b", version)
	}
	cmd.Args = append(cmd.Args, url, path)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		color.Red("Failed to clone git: %v", err)
	}
}

func installBinDep(url, path string) {
	filename := filepath.Base(url)
	destPath := filepath.Join(path, filename)
	resp, err := http.Get(url)
	if err != nil {
		color.Red("Failed to download bin: %v", err)
		return
	}
	defer resp.Body.Close()
	out, err := os.Create(destPath)
	if err != nil {
		color.Red("Failed to create file: %v", err)
		return
	}
	defer out.Close()
	if _, err := io.Copy(out, resp.Body); err != nil {
		color.Red("Failed to copy bin: %v", err)
		return
	}
	if err := os.Chmod(destPath, 0755); err != nil {
		color.Red("Failed to make executable: %v", err)
	}
}

func installTarDep(url, path string) {
	resp, err := http.Get(url)
	if err != nil {
		color.Red("Failed to download tar: %v", err)
		return
	}
	defer resp.Body.Close()
	gzr, err := gzip.NewReader(resp.Body)
	if err != nil {
		color.Red("Failed to gzip reader: %v", err)
		return
	}
	tr := tar.NewReader(gzr)
	for {
		header, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			color.Red("Failed to read tar: %v", err)
			return
		}
		target := filepath.Join(path, header.Name)
		switch header.Typeflag {
			case tar.TypeDir:
				os.MkdirAll(target, 0755)
			case tar.TypeReg:
				out, err := os.OpenFile(target, os.O_CREATE|os.O_RDWR, os.FileMode(header.Mode))
				if err != nil {
					color.Red("Failed to create file: %v", err)
					return
				}
				if _, err := io.Copy(out, tr); err != nil {
					color.Red("Failed to copy file: %v", err)
					out.Close()
					return
				}
				out.Close()
		}
	}
}

func installZipDep(url, path string) {
	resp, err := http.Get(url)
	if err != nil {
		color.Red("Failed to download zip: %v", err)
		return
	}
	defer resp.Body.Close()
	tmpFile, err := os.CreateTemp("", "nula-zip-*.zip")
	if err != nil {
		color.Red("Failed to create temp file: %v", err)
		return
	}
	defer os.Remove(tmpFile.Name())
	if _, err := io.Copy(tmpFile, resp.Body); err != nil {
		color.Red("Failed to copy zip: %v", err)
		return
	}
	tmpFile.Close()

	r, err := zip.OpenReader(tmpFile.Name())
	if err != nil {
		color.Red("Failed to open zip: %v", err)
		return
	}
	defer r.Close()
	for _, f := range r.File {
		fpath := filepath.Join(path, f.Name)
		if f.FileInfo().IsDir() {
			os.MkdirAll(fpath, os.ModePerm)
			continue
		}
		outFile, err := os.OpenFile(fpath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, f.Mode())
		if err != nil {
			color.Red("Failed to create file: %v", err)
			return
		}
		rc, err := f.Open()
		if err != nil {
			color.Red("Failed to open zip file: %v", err)
			outFile.Close()
			return
		}
		if _, err := io.Copy(outFile, rc); err != nil {
			color.Red("Failed to copy zip file: %v", err)
		}
		outFile.Close()
		rc.Close()
	}
}

func installHttpDep(url, path string) {
	filename := filepath.Base(url)
	destPath := filepath.Join(path, filename)
	resp, err := http.Get(url)
	if err != nil {
		color.Red("Failed to download http: %v", err)
		return
	}
	defer resp.Body.Close()
	out, err := os.Create(destPath)
	if err != nil {
		color.Red("Failed to create file: %v", err)
		return
	}
	defer out.Close()
	if _, err := io.Copy(out, resp.Body); err != nil {
		color.Red("Failed to copy http: %v", err)
		return
	}
}

func addToNulaToml(dep, version string) {
	configPath := "nula.toml"
	content, err := os.ReadFile(configPath)
	if err != nil {
		color.Red("Failed to read nula.toml: %v", err)
		return
	}
	lines := strings.Split(string(content), "\n")
	inDeps := false
	updated := false
	newLines := make([]string, 0)
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "[dependencies]" {
			inDeps = true
		}
		if inDeps && strings.HasPrefix(trimmed, dep+" =") {
			updated = true
			line = fmt.Sprintf("%s = \"%s\"", dep, version)
		}
		newLines = append(newLines, line)
		if inDeps && (strings.HasPrefix(trimmed, "[") && trimmed != "[dependencies]" || trimmed == "") {
			inDeps = false
		}
	}
	if !updated {
		newLines = append(newLines, fmt.Sprintf("%s = \"%s\"", dep, version))
	}
	if err := os.WriteFile(configPath, []byte(strings.Join(newLines, "\n")), 0644); err != nil {
		color.Red("Failed to write nula.toml: %v", err)
	}
	os.Setenv("NULA_PATH", filepath.Join(".", "nula_libs")+":"+os.Getenv("NULA_PATH"))
}
