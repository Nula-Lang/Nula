package main

import (
    "fmt"
    "os"
    "os/exec"
    "strings"

    "github.com/fatih/color"  // Dla kolorów
)

func main() {
    if len(os.Args) < 2 {
        help()
        return
    }

    cmd := os.Args[1]
    switch cmd {
    case "help", "?":
        help()
    case "build":
        runCmd("nula", "build")
    case "run":
        runCmd("nula-python", "main.nula")
    case "create":
        createProject()
    case "install":
        if len(os.Args) < 3 {
            errorMsg("No dep")
            return
        }
        installDep(os.Args[2])
    default:
        errorMsg("Unknown command")
    }
}

func help() {
    color.Cyan("Nula CLI Commands:")
    fmt.Println("  ? - Show help")
    fmt.Println("  build - Compile")
    fmt.Println("  run - Interpret")
    fmt.Println("  create <name> - New project")
    fmt.Println("  install <dep> - Add lib")
}

func runCmd(bin string, args ...string) {
    c := exec.Command(bin, args...)
    c.Stdout = os.Stdout
    c.Stderr = os.Stderr
    if err := c.Run(); err != nil {
        errorMsg(err.Error())
    } else {
        color.Green("Success!")
    }
}

func createProject() {
    name := "newproject"
    if len(os.Args) > 2 {
        name = os.Args[2]
    }
    os.Mkdir(name, 0755)
    os.WriteFile(name+"/main.nula", []byte('write("Hello Go!");'), 0644)
    color.Green("Created %s", name)
}

func installDep(dep string) {
    // Pobierz library.nula
    out, err := exec.Command("curl", "-s", "https://raw.githubusercontent.com/Nula-Lang/Nula/main/library.nula").Output()
    if err != nil {
        errorMsg("Curl failed")
        return
    }

    lines := strings.Split(string(out), "\n")
    var repo string
    for _, line := range lines {
        if strings.HasPrefix(line, dep+" -> ") {
            repo = strings.TrimPrefix(line, dep+" -> ")
            break
        }
    }
    if repo == "" {
        errorMsg("Dep not found")
        return
    }

    libDir := "/usr/lib/.nula-lib"
    os.MkdirAll(libDir, 0755)
    err = exec.Command("git", "clone", repo).Run()
    if err != nil {
        errorMsg("Git clone failed")
        return
    }
    color.Green("Installed %s", dep)
}

func errorMsg(msg string) {
    color.Red("ERROR: %s", msg)
    os.Exit(1)
}
