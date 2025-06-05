# 🗂️ Tooka

[![Cargo-Test](https://github.com/Benji377/tooka/actions/workflows/test.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/test.yml)
[![Rust-Clippy-Analyzer](https://github.com/Benji377/tooka/actions/workflows/clippy.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/clippy.yml)

<div align="center">
  <img src="./assets/logo-banner.png" alt="Tooka logo" style="width: 80%">
</div>

A fast, rule-based CLI tool for organizing files.

> [!WARNING]
> **Tooka is in early development.** Do *not* use it in production environments at this time.


---

## 🧭 Introduction

**Tooka** is a powerful file automation tool that helps you organize, rename, move, copy, compress, or delete files using rules written in simple YAML format.

You write the rules. Tooka does the sorting.

With a minimal CLI interface, Tooka enables automated file handling through filters like filename, extension, metadata, and file age.

---

## 🌐 Website

Visit [https://tooka.deno.dev](https://tooka.deno.dev) for a full overview.

👉 Try the **interactive rule builder** at [https://tooka.deno.dev/builder](https://tooka.deno.dev/builder) — generate a YAML rule and download it directly for use with Tooka!

---

## ✨ Features

- Define custom file rules in YAML
- Match by attributes like name, extension, size, metadata, and timestamps
- Perform actions: move, copy, rename, delete, skip
- Dry-run mode for safe previews
- Cross-platform: Windows, macOS, and Linux
- Shell autocompletion support

---

## 🚀 Getting Started

1. **Download Tooka** from the [releases page](https://github.com/Benji377/tooka/releases)
2. Run `tooka` in your terminal to verify it's installed
3. Create a new rule using the online builder or CLI:

   ```bash
   tooka template --help
   ```

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

## 📁 Rule Format

Tooka rules are written in YAML. Each rule defines when to match a file and what actions to take.

### ✅ Basic Structure

```yaml
id: example_rule
name: "Example Rule"
enabled: true
description: "Describe what this rule does"
priority: 1

when:
  any: false
  filename: "^.*\\.jpg$"
  extensions: ["jpg", "jpeg"]
  path: "**/holiday/**"
  size_kb:
    min: 10
    max: 5000
  mime_type: "image/jpeg"
  created_date:
    from: null
    to: null
  modified_date: null
  is_symlink: false
  metadata:
    - key: "EXIF:DateTime"
      value: null

then:
  - type: move
    to: "/path/to/destination"
    preserve_structure: false
  - type: rename
    to: "{{metadata.EXIF:DateTime|date:%Y-%m-%d}}-{{filename}}"
```

---

### 🔍 `when`: Conditions

Control how files are matched. Use `any: true` for OR logic, or omit for AND logic (default).

| Field           | Type              | Description                                        |
| --------------- | ----------------- | -------------------------------------------------- |
| `any`           | boolean           | `true` for OR logic between conditions             |
| `filename`      | string (regex)    | Match filename using regex                         |
| `extensions`    | list of strings   | File extensions (without the dot)                  |
| `path`          | string (glob)     | Glob pattern for file paths                        |
| `size_kb`       | object            | File size range in kilobytes: `min`, `max`         |
| `mime_type`     | string            | Match MIME type (e.g. `image/png`)                 |
| `created_date`  | object            | Match creation date range: `from`, `to` (ISO 8601) |
| `modified_date` | object or null    | Same as above, for modification time               |
| `is_symlink`    | boolean or null   | Match symlink status                               |
| `metadata`      | list of key/value | Match file metadata (e.g., EXIF)                   |

---

### ⚙️ `then`: Actions

Define what to do when the rule matches:

| Type     | Fields                                                    |
| -------- | --------------------------------------------------------- |
| `move`   | `to` (path), `preserve_structure` (bool)                  |
| `copy`   | `to` (path), `preserve_structure` (bool)                  |
| `delete` | `trash` (bool)                                            |
| `rename` | `to` (template string with variables like `{{filename}}`) |
| `skip`   | *(no fields)* — skips further rules for the current file  |

---

## 🐳 Try Tooka in Docker

Run Tooka inside a lightweight Debian container — perfect for testing rules in
isolation.

### Build the Docker Image

```bash
docker build -t tooka-playground .
```

### Start a Bash Session with Tooka Preinstalled

```bash
docker run --rm -it tooka-playground
```

Once inside the container, you can explore Tooka freely:

```bash
tooka --help
tooka config
tooka template
```

You can mount volumes to access your real files and rules:

```bash
docker run --rm -it \
  -v "$HOME/Downloads:/input" \
  -v "$PWD/rules:/rules" \
  tooka-playground
```

> 💡 Use `/input` as your working directory or test folder, and reference rules from `/rules`.

---

## ⚙️ Environment Variables

Tooka uses environment variables to override its default directories:

| Variable              | Description                                      |
| --------------------- | ------------------------------------------------ |
| `TOOKA_CONFIG_DIR`    | Custom path for Tooka’s config file              |
| `TOOKA_DATA_DIR`      | Custom path for data storage (e.g., rules, logs) |
| `TOOKA_SOURCE_FOLDER` | Custom path used by the sort command             |

Example usage:

```bash
export TOOKA_CONFIG_DIR="$HOME/.config/custom-tooka"
export TOOKA_DATA_DIR="$HOME/.local/share/custom-tooka"
export TOOKA_SOURCE_FOLDER="$HOME/downloads"
```

---

## 🛠️ CLI Commands

| Command               | Description                                        |
| --------------------- | -------------------------------------------------- |
| `add <YAML-file>`     | Import a rule file                                 |
| `remove <ID>`         | Remove a rule by its ID                            |
| `toggle <ID>`         | Enable or disable a rule                           |
| `list`                | List all available rules                           |
| `export <ID> <path>`  | Export a rule to file                              |
| `sort [OPTIONS]`      | Apply active rules to sort files                   |
| `template [OPTIONS]`  | Generate rule templates from predefined logic      |
| `config [OPTIONS]`    | View or modify Tooka’s configuration               |
| `completions <shell>` | Generate autocompletions for bash, zsh, fish, etc. |

---

## ⚙️ Configuration

Tooka uses a YAML configuration file stored in your system’s configuration directory. You can manage it entirely via the CLI:

```bash
tooka config --help
```

### What You Can Configure

| Field            | Description                                     |
| ---------------- | ----------------------------------------------- |
| `config_version` | Internal version tracking for config migrations |
| `source_folder`  | Default folder Tooka uses to sort files         |
| `rules_file`     | Path to the active YAML rule set                |
| `logs_folder`    | Directory where Tooka writes log files          |

### Config File Locations

* **Linux**: `~/.config/github.benji377/tooka/config.yaml`
* **Windows**: `%APPDATA%\github.benji377\tooka\config.yaml`
* **macOS**: `~/Library/Application Support/github.benji377/tooka/config.yaml`

---

## 📜 License

Tooka is open source and available under the [GPLv3 License](LICENSE).

---

## 💬 Support

Have a bug, idea, or question? Join the conversation in [GitHub Discussions](https://github.com/Benji377/tooka/discussions).
