package cmd

// Logic for tooka sort command, parses args/flags and calls sorter

import (
    "fmt"
    "github.com/spf13/cobra"
)

var (
    sourceFolder string
    ruleIDs      string
    dryRun       bool
)

var sortCmd = &cobra.Command{
    Use:   "sort",
    Short: "Manually runs the sorter on the source folder",
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Println("Running sort...")
        fmt.Printf("Source Folder: %s\n", sourceFolder)
        fmt.Printf("Rule IDs: %s\n", ruleIDs)
        fmt.Printf("Dry Run: %v\n", dryRun)
    },
}

func init() {
    sortCmd.Flags().StringVar(&sourceFolder, "source", "", "Override default source folder")
    sortCmd.Flags().StringVar(&ruleIDs, "rules", "", "Comma-separated rule IDs to run")
    sortCmd.Flags().BoolVar(&dryRun, "dry-run", false, "Simulate the sorting without making changes")
}