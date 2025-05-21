package cmd

// Exports a rule to a yaml a standalone yaml file by its ID.
import (
    "fmt"
    "github.com/spf13/cobra"
)

var (
    exportID    string
    outputPath  string
)

var exportCmd = &cobra.Command{
    Use:   "export",
    Short: "Exports a single rule by ID to a YAML file",
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Println("Exporting rule ID:", exportID)
        fmt.Println("Output path:", outputPath)
    },
}

func init() {
    exportCmd.Flags().StringVar(&exportID, "id", "", "ID of the rule to export")
    exportCmd.Flags().StringVar(&outputPath, "output", "", "Output file path")
    exportCmd.MarkFlagRequired("id")
    exportCmd.MarkFlagRequired("output")
}