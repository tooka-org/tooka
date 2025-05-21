package cmd

// Sets up the root command and global flags for the CLI application.

import (
	"fmt"
	"os"

    "github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
    Use:   "tooka",
    Short: "Tooka is an intelligent file sorter CLI",
    Long:  `Tooka is a rule-based file sorting tool for organizing cluttered folders manually.`,
}

func Execute() {
    if err := rootCmd.Execute(); err != nil {
        fmt.Println(err)
        os.Exit(1)
    }
}

func init() {
    rootCmd.AddCommand(sortCmd)
    rootCmd.AddCommand(configCmd)
    rootCmd.AddCommand(listCmd)
    rootCmd.AddCommand(addCmd)
    rootCmd.AddCommand(removeCmd)
    rootCmd.AddCommand(exportCmd)
}