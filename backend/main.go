package main

import (
	"fmt"
	"os"

	"nula-backend/internal/interpreter"
	"nula-backend/internal/parser"
)

func main() {
	if len(os.Args) < 3 || os.Args[1] != "run" {
		fmt.Println("Usage: nula-backend run <file.nula>")
		os.Exit(1)
	}
	filePath := os.Args[2]

	code, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("Error reading file: %v\n", err)
		os.Exit(1)
	}

	// Parse the code
	ast, err := parser.Parse(string(code))
	if err != nil {
		fmt.Printf("Parse error: %v\n", err)
		os.Exit(1)
	}

	// Interpret
	globalScope := interpreter.NewScope(nil)
	err = interpreter.Interpret(ast, globalScope)
	if err != nil {
		fmt.Printf("Runtime error: %v\n", err)
		os.Exit(1)
	}
}
