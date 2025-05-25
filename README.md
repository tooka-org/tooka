# Tooka

[![Cargo-Test](https://github.com/Benji377/tooka/actions/workflows/test.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/test.yml)
[![Rust-Clippy-Analyzer](https://github.com/Benji377/tooka/actions/workflows/clippy.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/clippy.yml)

<div align="center">
  <img src="./assets/logo-banner.png" alt="Tooka logo" style="width: 80%">
</div>

A fast, rule-based CLI tool for organizing your files.

> [!WARNING]
> Tooka lacks tests and is currently in early beta. Do **not** use it in production environments!

---

## Introduction

**Tooka** is a powerful file automation tool that helps you organize, rename, move, copy, compress, or delete files using rules written in simple YAML format.

You write the rules. Tooka does the sorting.

With a minimal CLI interface, Tooka enables automated file handling through filters like filename, extension, metadata, and file age.

---

## Features

* Define custom file rules in YAML
* Match by attributes like name, extension, size, metadata, and age
* Perform actions: move, copy, rename, delete, compress, skip
* Dry-run mode for safe previews
* Cross-platform: Windows, macOS, and Linux
* Shell autocompletion support

---

## Getting Started

1. **Download Tooka** from the [releases page](https://github.com/Benji377/tooka/releases)
2. Run `tooka` in your terminal to verify it's installed
3. Add rules via `tooka add` or place `.yaml` files in the rules directory:

   * **Linux**: `/home/<user>/.local/share/tooka`
   * **Windows**: `C:\Users\<user>\AppData\Roaming\github.benji377\tooka\data`
   * **macOS**: `/Users/<user>/Library/Application Support/io.github.benji377.tooka`

4. Run a dry-run to test your rules:

   ```bash
   tooka sort --dry-run ~/Downloads
    ```

> [!IMPORTANT]
> **Always run a dry-run first** to see what files would be moved, renamed, or deleted. Tooka **cannot recover lost or changed files**. Proceed carefully!

5. Once verified, run Tooka normally:

   ```bash
   tooka sort ~/Downloads
   ```

6. Explore all options with:

   ```bash
   tooka --help
   ```

---

## CLI Commands

| Command               | Description                                        |
| --------------------- | -------------------------------------------------- |
| `add <YAML-file>`     | Import a rule file                                 |
| `remove <ID>`         | Remove a rule by its ID                            |
| `toggle <ID>`         | Enable or disable a rule                           |
| `list`                | List all available rules                           |
| `export <ID> <path>`  | Export a rule to file                              |
| `sort [OPTIONS]`      | Apply active rules to sort files                   |
| `config [OPTIONS]`    | View or modify Tooka’s configuration               |
| `completions <shell>` | Generate autocompletions for bash, zsh, fish, etc. |

---

## Rule Structure

Here's an example rule:

```yaml
id: "img-date-001"
name: "Organize Images by Date"
enabled: true

match:
  extensions: [".jpg", ".jpeg", ".png"]
  mime_type: "image/*"
  pattern: "IMG_*"
  metadata:
    exif_date: true
    fields:
      - key: "EXIF:Model"
        pattern: "iPhone*"
  conditions:
    older_than_days: 7
    size_greater_than_kb: 100
    filename_regex: "^IMG_.*"

actions:
  - type: move
    destination: "~/Pictures/Organized"
    path_template:
      source: "exif"
      format: "{year}/{month}/{day}"
    rename_template: "{year}-{month}_{filename}.{ext}"
    create_dirs: true
```

---

### Match Fields

| Field                | Description                                                 |
| -------------------- | ----------------------------------------------------------- |
| `extensions`         | File extension filter (e.g., `.jpg`, `.pdf`)                |
| `mime_type`          | Match MIME types like `image/*`                             |
| `pattern`            | Simple filename prefix matching (e.g., `IMG_`)              |
| `filename_regex`     | Advanced regex pattern matching on filenames                |
| `metadata.exif_date` | Use EXIF date (e.g., for images)                            |
| `metadata.fields`    | Match EXIF or custom metadata by key/pattern                |
| `conditions`         | Add constraints: age, size, symlink status, ownership, etc. |

---

### Matching Logic

* The **first** matching rule is applied
* If no rule matches, the file is left untouched
* Use `enabled: false` to temporarily disable rules without deleting them

---

## Actions

| Action Type    | Description                         |
| -------------- | ----------------------------------- |
| `move`, `copy` | Move or duplicate the file          |
| `rename`       | Rename file using a template        |
| `compress`     | Compress the file using GZIP        |
| `delete`       | Delete the file                     |
| `skip`         | Skip file without taking any action |

Optional fields:

* `destination`: Target directory
* `path_template`: Dynamic folder structure, e.g., based on date
* `rename_template`: Use variables like `{filename}`, `{year}`, `{ext}`
* `create_dirs`: Automatically create missing folders

---

## Configuration

Tooka uses a YAML configuration file stored in your system’s configuration directory. You can manage it entirely via the CLI:

```bash
tooka config --help
```

### What You Can Configure

The configuration file includes:

| Field                | Description                                          |
| -------------------- | ---------------------------------------------------- |
| `config_version`     | Internal version tracking for config migrations      |
| `source_folder`      | Default folder Tooka uses to sort files              |
| `rules_file`         | Path to the active YAML rule set                     |
| `logs_folder`        | Directory where Tooka writes log files               |

These values are saved automatically when running Tooka. You can also manage them manually or programmatically using the API.

> 💡 Default values are initialized based on platform-specific directories like `~/Downloads` and system data paths (via `ProjectDirs` and `UserDirs`).

---

### Config File Behavior

Tooka handles config loading and saving automatically:

* On first run, if no config file exists, a default is created and saved
* All config values are serialized/deserialized as YAML
* Config location:

  * **Linux**: `~/.config/github.benji377/tooka/config.yaml`
  * **Windows**: `%APPDATA%\github.benji377\tooka\config.yaml`
  * **macOS**: `~/Library/Application Support/github.benji377/tooka/config.yaml`

---

## License

Tooka is open source and available under the [GPLv3 License](LICENSE).

---

## Support

Have a bug, idea, or question? Join the conversation in [GitHub Discussions](https://github.com/Benji377/tooka/discussions).
