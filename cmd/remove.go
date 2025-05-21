package cmd

// Removes a rule from the rules file by its ID.
import (
    "fmt"
    "github.com/spf13/cobra"
)

var removeCmd = &cobra.Command{
    Use:   "remove",
    Short: "Removes a single rule by ID",
	Args: cobra.ExactArgs(1),
    Run: func(cmd *cobra.Command, args []string) {
		rule_id := args[0]
        fmt.Println("Removing rule ID:", rule_id)
    },
}