# Tooka — Intelligent File Sorter CLI

---

## Table of Contents
1. [Project Overview](#project-overview)  
2. [Features & Goals](#features--goals)  
3. [Configuration](#configuration)  
   - [Global Config](#global-config)  
   - [Rules File](#rules-file)  
4. [Rule Schema](#rule-schema)  
5. [CLI Commands](#cli-commands)  
6. [Logging & Statistics](#logging--statistics)  
7. [User Interaction Flow](#user-interaction-flow)  
8. [Dependencies & Technical Choices](#dependencies--technical-choices)  
9. [Future Roadmap](#future-roadmap)  

---

## Project Overview

**Tooka** is a cross-platform CLI tool (Linux & Windows) to help users organize files in a specified folder using customizable, flexible rules.  
It enables manual sorting of files based on metadata, patterns, file properties, and dynamic destination paths, improving users’ digital well-being by keeping their folders clean and organized.

Tooka focuses on:  
- Flexibility: handle all file types with rich, metadata-aware rules  
- User control: manual triggering of sorting  
- Simplicity: CLI-based, no complicated GUIs or background watchers  
- Extensibility: easy to add new filters and actions  

---

## Features & Goals

- Sort files manually on demand (`tooka sort`)  
- Flexible, YAML-defined rules with metadata-driven matching and dynamic path templating  
- Single configurable source folder, overridable via CLI  
- Statistics tracking (files moved, folder count) stored in JSON  
- Logging of file moves and operations in a structured log file  
- CLI commands for adding/removing/importing/exporting rules by ID  
- Dry-run stats to preview sorting impact  
- Cross-platform support: Windows & Linux (macOS excluded)  
- Minimal dependencies, clean codebase in Go  
- No background file watching; sorting runs only when triggered  

---

## Configuration

### Global Config

Located at:  
`~/.config/tooka/config.yaml` (Linux)  
`%APPDATA%\tooka\config.yaml` (Windows)  

Contains:  
- `source_folder`: default folder to sort (e.g., `~/Downloads`)
- `version`: version of the app
- `rules_file`: path to the single YAML rules file  
- `logs_folder`: path for logging actions  
- `first_run_complete`: boolean to skip prompts on startup  

**On first run:**  
- Prompts user to input `source_folder` and creates default config  
- Creates empty rules file if none exists  

---

### Rules File

Located by default at:  
`~/.config/tooka/rules.yaml` (Linux)  
`%APPDATA%\tooka\rules.yaml` (Windows)  

- Contains **all sorting rules** in a single YAML file  
- Each rule has a unique `id` (UUID or short string)  
- Users manage rules by importing/exporting full YAML snippets  
- Overwriting a rule requires full replacement (no partial edits)  

---

## Rule Schema

```yaml
rules:
  - id: "img-date-001"
    name: "Organize Images by Date"
    enabled: true

    match:
      extensions: [".jpg", ".jpeg", ".png"]
      mime_type: "image/*"
      pattern: "IMG_*"
      metadata:
        exif_date: true
      conditions:
        older_than_days: 7
        size_greater_than_kb: 100

    action:
      type: move
      destination: "~/Pictures/Organized"
      path_template:
        source: "exif"          # Options: exif | mtime | ctime
        format: "{year}/{month}/{day}"
      rename_template: "{year}-{month}_{filename}.{ext}"
      create_dirs: true

    flags:
      dry_run: false
```

### Explanation:

* `id`: unique rule identifier for import/export and referencing
* `name`: descriptive name shown in CLI
* `enabled`: toggle rule on/off
* `match`: flexible conditions to match files, supports:

  * `extensions` (list of file suffixes)
  * `mime_type` (content-based type matching, supports wildcards)
  * `pattern` (glob matching on filename)
  * `metadata` (e.g., use EXIF date)
  * `conditions` (age, size constraints)
* `action`: defines what happens to matched files

  * `type`: move | copy | delete | skip | rename
  * `destination`: base path for file actions
  * `path_template`: dynamic subfolders based on timestamps
  * `rename_template`: rename output files dynamically
  * `create_dirs`: auto-create folders if needed
* `flags`: e.g., `dry_run` to simulate actions

---

## CLI Commands

### 1. `tooka sort`

Manually runs the sorter on the source folder.

**Usage:**

```bash
tooka sort [--source <folder>] [--rules <rule_id,...>] [--dry-run]
```

* `--source`: override default source folder for this run
* `--rules`: comma-separated rule IDs to run (defaults to all enabled rules)
* `--dry-run`: do not move files, only simulate and output stats

---

### 2. `tooka stats`

Shows statistics of past sorting runs.

**Usage:**

```bash
tooka stats
```

* Reads JSON stats file and summarizes:

  * Number of files sorted
  * Number of folders created
  * Most used rules by file count
  * Last sort timestamp

---

### 3. `tooka list`

Lists all current rules with their metadata.

**Usage:**

```bash
tooka list
```

* Displays rule IDs, names, enabled status, and match summary

---

### 4. `tooka add`

Adds a new rule by importing a YAML snippet file.

**Usage:**

```bash
tooka add --file <rule_file.yaml>
```

* Merges new rule into rules file
* Validates schema and checks duplicate IDs

---

### 5. `tooka remove`

Removes a rule by its ID.

**Usage:**

```bash
tooka remove --id <rule_id>
```

* Deletes the rule from the rules file

---

### 6. `tooka export`

Exports a single rule by ID to a YAML file.

**Usage:**

```bash
tooka export --id <rule_id> --output <path.yaml>
```

---

## Logging & Statistics

* Actions (file moves, deletes) logged with [zerolog](https://github.com/rs/zerolog)
* Log file rotated via [lumberjack](https://github.com/natefinch/lumberjack)
* Logs contain JSON lines with timestamp, source, destination, rule ID, action
* Stats stored in a JSON file, capturing per-run summaries:

  * Timestamp
  * Number of files moved
  * Number of folders created
  * List of rules triggered and counts

---

## User Interaction Flow

1. **First run:**

   * Prompt user for default source folder
   * Create global config and empty rules file

2. **Adding rules:**

   * User creates or imports rules via YAML files
   * `tooka add --file myrule.yaml` merges into config

3. **Sorting files:**

   * User runs `tooka sort` manually to organize files
   * Optionally uses `--rules` to run specific subsets
   * Output summarizes actions; logs and stats updated

4. **Viewing stats:**

   * `tooka stats` shows history of sorting actions

5. **Managing rules:**

   * `tooka list` to see current rules
   * `tooka remove --id <id>` to delete rules
   * `tooka export` to save rules for sharing

---

## Dependencies & Technical Choices

* Language: Go (for easy cross-compilation)
* Config & Rules parsing: `gopkg.in/yaml.v3`
* EXIF parsing: `github.com/rwcarlsen/goexif/exif`
* Logging: `github.com/rs/zerolog` + `github.com/natefinch/lumberjack` for log rotation
* CLI framework: `spf13/cobra` for robust CLI commands and flags
* Stats persistence: local JSON files for simplicity and readability

---

## Future Roadmap

* Add support for multiple source folders
* Background file watching with optional daemon mode
* Extended actions (compress, encrypt, upload)
* TUI for rule management
* Rule validation and linting tools
* Plugin system for custom matchers/actions
* More metadata support (e.g., PDF metadata, EXIF GPS)

---

# Summary

Tooka is a flexible, user-driven file sorter designed to help users regain control over their digital clutter via manual, rule-based sorting.
With a focus on usability and extensibility, it balances power and simplicity in a clean CLI interface, perfect for Linux and Windows users.


---

# CONSIDER

- Terminal Colors: https://github.com/console-rs/indicatif
- Terminal Design: https://github.com/crossterm-rs/crossterm