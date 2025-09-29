package cli

import (
    "fmt"
    "os"
    "os/exec"
    "path/filepath"

    "github.com/fatih/color"
    "nula-go/deps"
)

// RunCLI executes the CLI based on provided arguments.
func RunCLI() {
    if len(os.Args) < 2 {
        showHelp()
        os.Exit(1)
    }

    cmd := os.Args[1]
    switch cmd {
    case "help", "?":
        showHelp()
    case "build":
        executeCommand("nula", []string{"build"})
    case "run":
        executeCommand("nula-python", []string{"main.nula"})
    case "create":
        createProject()
    case "install":
        if len(os.Args) < 3 {
            errorMsg("No dependency specified")
            return
        }
        deps.InstallDependency(os.Args[2])
    default:
        errorMsg(fmt.Sprintf("Unknown command: %s", cmd))
    }
}

// showHelp displays the help message with available commands.
func showHelp() {
    color.Cyan("Nula CLI Commands:")
    fmt.Println("  ", color.YellowString("?"), "        - Show this help")
    fmt.Println("  ", color.YellowString("build"), "    - Compile code to binary")
    fmt.Println("  ", color.YellowString("run"), "      - Run .nula file in interpreted mode")
    fmt.Println("  ", color.YellowString("create"), "   - Create a new project")
    fmt.Println("  ", color.YellowString("install"), "  - Install a dependency (e.g., install the-racer)")
}

// executeCommand runs an external command with arguments and handles output.
func executeCommand(bin string, args []string) {
    cmd := exec.Command(bin, args...)
    cmd.Stdout = os.Stdout
    cmd.Stderr = os.Stderr
    if err := cmd.Run(); err != nil {
        errorMsg(fmt.Sprintf("Command '%s %v' failed: %v", bin, args, err))
    } else {
        color.Green("Success!")
    }
}

// createProject creates a new project directory with a sample main.nula file.
func createProject() {
    projectName := "newproject"
    if len(os.Args) > 2 {
        projectName = os.Args[2]
    }

    projectPath := filepath.Join(".", projectName)
    if err := os.Mkdir(projectPath, 0755); err != nil {
        errorMsg(fmt.Sprintf("Failed to create project directory: %v", err))
        return
    }

    mainNula := filepath.Join(projectPath, "main.nula")
    sampleCode := []byte(`write("Hello Nula!"); @ Sample Nula program
x = "world";
write(x);
`)
    if err := os.WriteFile(mainNula, sampleCode, 0644); err != nil {
        errorMsg(fmt.Sprintf("Failed to create main.nula: %v", err))
        return
    }

    color.Green("Created project: %s", projectName)
}

// errorMsg prints an error message in red and exits.
func errorMsg(msg string) {
    color.Red("ERROR: %s", msg)
    os.Exit(1)
}
