package main

import (
	"time"

	"github.com/charmbracelet/bubbletea"
)

type spinnerModel struct{}
type spinnerDone struct{}

func (m spinnerModel) Init() tea.Cmd {
	return tickCmd()
}

func (m spinnerModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg.(type) {
		case spinnerDone:
			return m, tea.Quit
		case tea.KeyMsg:
			return m, tea.Quit
		default:
			return m, tickCmd()
	}
}

func (m spinnerModel) View() string {
	return "Installing... (press any key to quit)"
}

func tickCmd() tea.Cmd {
	return tea.Tick(time.Second/10, func(time.Time) tea.Msg {
		return nil
	})
}
