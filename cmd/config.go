package cmd

// Allows reading, checking the location and writing the config file.

import (
    "errors"
    "fmt"

    "github.com/spf13/cobra"
)

var (
    locateConfig bool
    initConfig   bool
    resetConfig  bool
    showConfig   bool
)

var configCmd = &cobra.Command{
    Use:     "config",
    Short:   "Manages the Tooka configuration file",
    Long:    "Allows inspecting, initializing, and resetting the Tooka configuration",
    RunE: func(cmd *cobra.Command, args []string) error {
        // Count how many flags are true
        flagCount := 0
        if locateConfig {
            flagCount++
        }
        if initConfig {
            flagCount++
        }
        if resetConfig {
            flagCount++
        }
        if showConfig {
            flagCount++
        }

        if flagCount == 0 {
            return errors.New("no action specified — use one of: --locate, --init, --reset, --show")
        }
        if flagCount > 1 {
            return errors.New("only one flag can be used at a time: choose from --locate, --init, --reset, --show")
        }

        switch {
        case locateConfig:
            fmt.Println("Config file is located at: <path/to/config.yaml>")
        case initConfig:
            fmt.Println("Initializing config file...")
        case resetConfig:
            fmt.Println("Resetting config to default...")
        case showConfig:
            fmt.Println("Current config contents:\n---\n<YAML output here>")
        }

        return nil
    },
}

func init() {
    configCmd.Flags().BoolVar(&locateConfig, "locate", false, "Print the location of the config file")
    configCmd.Flags().BoolVar(&initConfig, "init", false, "Initialize config file if it doesn't exist")
    configCmd.Flags().BoolVar(&resetConfig, "reset", false, "Reset the config file to default values")
    configCmd.Flags().BoolVar(&showConfig, "show", false, "Display the current config file contents")

    rootCmd.AddCommand(configCmd)
}
