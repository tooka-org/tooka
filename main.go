package main

import (
	"fmt"
	"os"

	"github.com/Benji377/tooka/internal/core"
	"github.com/Benji377/tooka/cmd"
)

func main() {
	// 1. Load or create config (prompts user on first run)
	config, err := core.LoadConfig()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to load config: %v\n", err)
		os.Exit(1)
	}

	// 2. Initialize logger with logs folder from config
	core.InitLogger(config.LogsFolder)

	// 3. Execute CLI commands
	cmd.Execute()
}
