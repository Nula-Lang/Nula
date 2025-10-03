package main

import (
	"fmt"
	"strings"

	"github.com/fatih/color"
)

func searchDeps(query string) {
	index := loadIndex()
	color.Cyan("Search results for %s:", query)
	found := false
	for dep, info := range index {
		if strings.Contains(strings.ToLower(dep), strings.ToLower(query)) {
			fmt.Printf("%s (%s, %s)\n", dep, info.Type, info.URL)
			found = true
		}
	}
	if !found {
		fmt.Println("No results found")
	}
}
