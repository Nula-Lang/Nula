// nula-backend.go - Expanded Backend in Go for parser/interpreter
// Handles more syntax: variables, basic math, if statements, loops.
// Indentation-based blocks like Python.
// Compile: go build -o ~/.nula/bin/nula-backend nula-backend.go
// Cross-platform considerations added.

package main

import (
	"bufio"
	"fmt"
	"io/ioutil"
	"math"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
)

type Scope struct {
	vars map[string]interface{}
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: nula-backend run <file.nula> [args]")
		os.Exit(1)
	}
	command := os.Args[1]
	if command != "run" {
		fmt.Printf("Unknown command: %s\n", command)
		os.Exit(1)
	}
	file := os.Args[2]

	code, err := ioutil.ReadFile(file)
	if err != nil {
		fmt.Printf("Error reading file: %v\n", err)
		os.Exit(1)
	}

	// Parse and interpret
	globalScope := &Scope{vars: make(map[string]interface{})}
	interpret(string(code), globalScope)
}

func interpret(code string, scope *Scope) {
	lines := strings.Split(code, "\n")
	inMultiComment := false
	libs := make(map[string]bool)
	importedModules := []string{}
	blockStack := []string{} // For if, loop blocks
	currentIndent := 0
	var blockCode strings.Builder

	for i, line := range lines {
		trimmed := strings.TrimSpace(line)
		indentLevel := len(line) - len(strings.TrimLeft(line, " \t"))

		if trimmed == "" {
			continue
		}

		// Handle block end
		if len(blockStack) > 0 && indentLevel < currentIndent {
			// Execute block
			blockType := blockStack[len(blockStack)-1]
			switch blockType {
			case "if":
				// Eval condition from previous line, but simplified
				// For now, assume true
				interpret(blockCode.String(), scope)
			case "loop":
				// Simple infinite loop for demo
				for {
					interpret(blockCode.String(), scope)
				}
			}
			blockStack = blockStack[:len(blockStack)-1]
			blockCode.Reset()
		}

		if inMultiComment {
			if strings.Contains(trimmed, "!") {
				inMultiComment = false
			}
			continue
		}

		if strings.HasPrefix(trimmed, "!") {
			inMultiComment = true
			continue
		}

		if strings.HasPrefix(trimmed, "@") {
			continue
		}

		// Imports
		if strings.HasPrefix(trimmed, ":") && strings.HasSuffix(trimmed, ":") {
			lib := strings.Trim(trimmed, ":")
			libs[lib] = true
			continue
		}
		if strings.HasPrefix(trimmed, "<") && strings.HasSuffix(trimmed, ">") {
			module := strings.Trim(trimmed, "<>")
			importedModules = append(importedModules, module)
			continue
		}

		// Embedded
		embeddedRe := regexp.MustCompile(`# =(\w+)= \[(.*)\]`)
		if match := embeddedRe.FindStringSubmatch(trimmed); match != nil {
			lang := match[1]
			embeddedCode := match[2]
			switch lang {
			case "python":
				tmpFile, _ := ioutil.TempFile("", "embedded*.py")
				tmpFile.Write([]byte(embeddedCode))
				tmpFile.Close()
				cmd := exec.Command("python", tmpFile.Name())
				output, _ := cmd.CombinedOutput()
				fmt.Print(string(output))
				os.Remove(tmpFile.Name())
			}
			continue
		}

		// write
		if strings.HasPrefix(trimmed, "write ") {
			str := strings.TrimPrefix(trimmed, "write ")
			str = strings.Trim(str, "\"")
			fmt.Println(evalExpr(str, scope)) // Support expr
			continue
		}

		// Variable assignment: name = value
		if strings.Contains(trimmed, " = ") {
			parts := strings.SplitN(trimmed, " = ", 2)
			name := parts[0]
			value := evalExpr(parts[1], scope)
			scope.vars[name] = value
			continue
		}

		// If statement: if cond
		if strings.HasPrefix(trimmed, "if ") {
			cond := strings.TrimPrefix(trimmed, "if ")
			// Eval cond, simplified to true if non-empty
			if evalExpr(cond, scope) != nil {
				blockStack = append(blockStack, "if")
				currentIndent = indentLevel + 1 // Expect indent
			} else {
				// Skip to end of block
				for j := i + 1; j < len(lines); j++ {
					if (len(lines[j]) - len(strings.TrimLeft(lines[j], " \t"))) <= indentLevel {
						break
					}
				}
			}
			continue
		}

		// Loop: loop
		if trimmed == "loop" {
			blockStack = append(blockStack, "loop")
			currentIndent = indentLevel + 1
			continue
		}

		// If in block, add to blockCode
		if len(blockStack) > 0 && indentLevel >= currentIndent {
			blockCode.WriteString(line + "\n")
			continue
		}

		// Default: treat as expr
		evalExpr(trimmed, scope)
	}

	// Load libs (simulate)
	for lib := range libs {
		fmt.Printf("Loaded lib %s\n", lib)
	}
	for _, mod := range importedModules {
		fmt.Printf("Imported %s\n", mod)
	}
}

// Simple expr eval: support numbers, vars, + - * / ^
func evalExpr(expr string, scope *Scope) interface{} {
	expr = strings.TrimSpace(expr)
	if val, err := strconv.ParseFloat(expr, 64); err == nil {
		return val
	}
	if val, ok := scope.vars[expr]; ok {
		return val
	}
	// Basic ops
 ops := []struct {
		op string
		f  func(float64, float64) float64
	}{
		{"+", func(a, b float64) float64 { return a + b }},
		{"-", func(a, b float64) float64 { return a - b }},
		{"*", func(a, b float64) float64 { return a * b }},
		{"/", func(a, b float64) float64 { return a / b }},
		{"^", math.Pow},
	}
	for _, op := range ops {
		if strings.Contains(expr, op.op) {
			parts := strings.SplitN(expr, op.op, 2)
			a := evalExpr(parts[0], scope).(float64)
			b := evalExpr(parts[1], scope).(float64)
			return op.f(a, b)
		}
	}
	return expr // String
}
