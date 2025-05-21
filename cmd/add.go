package cmd

// Adds/imports a rule from a yaml file.
import (
	"fmt"

	"github.com/spf13/cobra"
)

var addCmd = &cobra.Command{
    Use:   "add <file>",
    Short: "Adds a new rule by importing a YAML snippet file",
	Long:  `Adds a new rule by importing a YAML snippet file. The file should contain the rule definition in YAML format.`,
	Args: cobra.ExactArgs(1),
    Run: func(cmd *cobra.Command, args []string) {
        filePath := args[0]
        fmt.Println("Adding rule from file:", filePath)
    },
}