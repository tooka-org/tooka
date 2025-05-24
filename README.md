#  Tooka

<div align="center">
  <img src="./assets/logo-banner.png" alt="Tooka logo" style="max-width: 600px;">
</div>

> [!WARNING]
> Tooka lacks tests and is therefore still in early beta. Do not use in production!

## 📦 Introduction

**Tooka** is a powerful file automation tool that helps you organize, rename, move, copy, and manage files based on custom rules defined in YAML format.

You write rules. Tooka sorts files.

With a simple CLI, Tooka enables automated file handling through matching criteria like filename, extension, metadata, and file age.

---

## ⚙️ Features

* Define rules using YAML
* Match by file attributes, EXIF metadata, name patterns, and conditions
* Perform actions like move, copy, compress, delete, skip, and rename
* Dry-run mode for testing
* Works across Windows, macOS, and Linux
* Shell autocompletions

---

## 🚀 Getting Started

1. **Download Tooka** from the [releases page](https://github.com/Benji377/tooka/releases)
2. Run `tooka` in your terminal to confirm it's working
3. Create or import rules with `tooka add` or by placing YAML files in the rules directory:

   * **Linux**: `/home/<user>/.local/share/tooka`
   * **Windows**: `C:\Users\<user>\AppData\Roaming\github.benji377\tooka\data`
   * **macOS**: `/Users/<user>/Library/Application Support/io.github.benji377.tooka`
4. Run `tooka sort` to apply your rules to a folder

> [!IMPORTANT]
> **Always perform a dry-run first** using `tooka sort --dry-run` to review what files will be moved, renamed, or deleted. Tooka is not responsible for lost or changed files. Proceed carefully!


5. Explore all options with `tooka --help`

---

## 🛠️ CLI Commands

| Command               | Description                                 |
| --------------------- | ------------------------------------------- |
| `add <YAML-file>`     | Import a rule file                          |
| `remove <ID>`         | Remove a rule by its ID                     |
| `toggle <ID>`         | Enable/disable a rule                       |
| `list`                | List all available rules                    |
| `export <ID> <path>`  | Export a rule to file                       |
| `sort [OPTIONS]`      | Apply rules to sort files                   |
| `config [OPTIONS]`    | Show or edit configuration                  |
| `completions <Shell>` | Generate shell completions (bash, zsh, etc) |

---

## 🧠 Rule Structure

Here’s an example rule:

```yaml
rules:
  - id: "img-date-001"
    name: "Organize Images by Date"
    enabled: true

    match:
      extensions: [".jpg", ".jpeg", ".png"]
      mime_type: "image/*"
      pattern: "IMG_*" # simple prefix match
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

### 🔍 Match Fields

| Field                | Description                                                              |
| -------------------- | ------------------------------------------------------------------------ |
| `extensions`         | Match by file extension (e.g., `.jpg`)                                   |
| `mime_type`          | Match MIME types like `image/*`                                          |
| `pattern`            | Simple prefix string matching on filenames (e.g., `IMG_`)                |
| `filename_regex`     | Full regular expression for advanced filename matching                   |
| `metadata.exif_date` | Use EXIF date for sorting                                                |
| `metadata.fields`    | Match specific metadata fields using `key` + `pattern`                   |
| `conditions`         | Logical conditions like file age, size, owner, symlink status, and regex |


### 🪄 Matching Logic

* The **first** matching rule is applied.
* If no rule matches, the file is left untouched.
* Use `enabled: false` to temporarily disable a rule.

---

## ✏️ Actions

| Type           | Description                    |
| -------------- | ------------------------------ |
| `move`, `copy` | Relocate or duplicate the file |
| `rename`       | Change the filename            |
| `compress`     | Compress the file (GZIP)       |
| `delete`       | Delete the file                |
| `skip`         | Skip file without action       |

Each action can be enhanced with:

* `destination`: Target directory
* `path_template`: Dynamic folder structure (e.g., by EXIF date)
* `rename_template`: Use variables like `{filename}`, `{year}`, `{ext}`
* `create_dirs`: Create missing directories if needed

---

## 🔧 Configuration

To view or edit Tooka’s configuration, run:

```bash
tooka config --help
```

You can set paths for rules, default folders, and more.

---

## 📄 License

Tooka is open source and licensed under the [GPLv3 License](LICENSE).

---

## 🙋 Support

For bugs, features, or discussions, visit the [GitHub Discussions](https://github.com/Benji377/tooka/discussions).