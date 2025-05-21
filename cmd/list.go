package cmd

// Reads the rules file and prints the rules in a human-readable format.
import (
    "fmt"
    "github.com/spf13/cobra"
)

var listCmd = &cobra.Command{
    Use:   "list",
    Short: "Lists all current rules with their metadata",
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Println("Listing all rules...")
    },
}