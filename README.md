# 🗂️ Tooka

[![clippy]](https://github.com/tooka-org/tooka/actions/workflows/clippy.yml)
[![test]](https://github.com/tooka-org/tooka/actions/workflows/test.yml)
[![feedback]](https://tally.so/r/mBVyLe)

[clippy]: https://img.shields.io/github/actions/workflow/status/tooka-org/tooka/clippy.yml?label=Clippy&logo=rust&style=for-the-badge&labelColor=555555
[test]: https://img.shields.io/github/actions/workflow/status/tooka-org/tooka/test.yml?label=Tests&logo=githubactions&style=for-the-badge&labelColor=555555
[feedback]: https://img.shields.io/badge/feedback-Tally-blueviolet?style=for-the-badge&labelColor=555555&logo=googleforms

<div align="center">
  <img src="./assets/logo-banner.png" alt="Tooka logo" style="width: 80%">
</div>

A fast, rule-based CLI tool for organizing files.

---

## 🧭 Introduction

**Tooka** is a flexible command-line tool for automating your filesystem: organize, rename, move, copy, or delete files using simple, powerful YAML rules.

You define what files to match (by name, extension, metadata, size, etc.) and what should happen to them — Tooka handles the rest with blazing-fast parallel processing.

---

## ✨ Features

* **🎯 Rule-based automation** - Define custom file organization rules using declarative YAML
* **⚡ High-performance** - Parallel recursive directory traversal and file operations
* **🔍 Flexible filtering** - Match files by name patterns, extensions, MIME types, size, metadata, and timestamps
* **🛠️ Multiple actions** - Move, copy, rename, delete, or skip files based on conditions
* **📝 Template support** - Dynamic file naming with customizable templates
* **🔒 Safe operations** - Dry-run mode and comprehensive logging for safety
* **🌐 Cross-platform** - Works seamlessly on Windows, macOS, and Linux

---

## 🚀 Quick Start

1. **Install Tooka**:
   ```bash
   # From source
   cargo install --git https://github.com/tooka-org/tooka
   
   # Or download pre-built binaries from releases
   ```

2. **Initialize configuration**:
   ```bash
   tooka config init
   ```

3. **Add your first rule**:
   ```bash
   # Create a simple rule to organize images
   echo 'id: organize-images
   priority: 100
   enabled: true
   when:
     extensions: ["jpg", "jpeg", "png", "gif"]
   then:
     - action: move
       destination: "~/Pictures/{{extension}}"' > image-rule.yaml
   
   tooka add image-rule.yaml
   ```

4. **Test with dry-run**:
   ```bash
   tooka sort --dry-run ~/Downloads
   ```

5. **Apply the rules**:
   ```bash
   tooka sort ~/Downloads
   ```

---

## 📖 Usage Examples

### Basic File Organization

```yaml
# Organize downloads by file type
id: organize-downloads
priority: 100
enabled: true
when:
  extensions: ["pdf", "doc", "docx"]
then:
  - action: move
    destination: "~/Documents/{{filename}}"
```

### Size-based Filtering

```yaml
# Move large files to external storage
id: archive-large-files
priority: 200
enabled: true
when:
  size_kb:
    min: 102400  # Files larger than 100MB
then:
  - action: move
    destination: "/external/storage/{{filename}}"
```

### Date-based Organization

```yaml
# Organize photos by creation date
id: organize-photos-by-date
priority: 150
enabled: true
when:
  mime_type: "image/*"
  created_date:
    from: "2024-01-01"
    to: "2024-12-31"
then:
  - action: move
    destination: "~/Photos/{{created_year}}/{{created_month}}/{{filename}}"
```

---

## 🔧 CLI Commands

### Core Commands

- `tooka sort [PATH]` - Sort files in a directory using defined rules
- `tooka add <RULE_FILE>` - Add a new rule from a YAML file
- `tooka list` - List all configured rules
- `tooka remove <RULE_ID>` - Remove a rule by its ID
- `tooka toggle <RULE_ID>` - Enable/disable a rule

### Configuration

- `tooka config init` - Initialize Tooka configuration
- `tooka config show` - Show current configuration
- `tooka config set <KEY> <VALUE>` - Set configuration values

### Utilities

- `tooka export <RULE_ID> [FILE]` - Export a rule to a file
- `tooka validate` - Validate all rules for syntax errors
- `tooka template` - Generate rule templates
- `tooka completions <SHELL>` - Generate shell completions

---

## 🎛️ Rule Structure

Rules are defined in YAML format with the following structure:

```yaml
id: unique-rule-identifier          # Required: Unique identifier
priority: 100                       # Required: Execution priority (higher = first)
enabled: true                       # Required: Enable/disable rule
when:                               # Required: Conditions to match files
  filename: "pattern.*"             # Optional: Regex pattern for filename
  extensions: ["jpg", "png"]        # Optional: File extensions
  path: "*/Downloads/*"             # Optional: Path pattern
  size_kb:                          # Optional: Size range in KB
    min: 1024
    max: 10240
  mime_type: "image/*"              # Optional: MIME type pattern
  created_date:                     # Optional: Creation date range
    from: "2024-01-01"
    to: "2024-12-31"
  modified_date:                    # Optional: Modification date range
    from: "2024-01-01"
    to: "2024-12-31"
  is_symlink: false                 # Optional: Match symlinks
  metadata:                         # Optional: EXIF/metadata fields
    - key: "Camera"
      value: "Canon*"
  any: false                        # Optional: Use OR logic instead of AND
then:                               # Required: Actions to perform
  - action: move                    # Action type: move, copy, rename, delete, skip
    destination: "~/target/{{filename}}"  # Destination with template support
```

---

## 📝 Template Variables

Tooka supports dynamic templates for file destinations:

| Variable | Description | Example |
|----------|-------------|---------|
| `{{filename}}` | Full filename with extension | `document.pdf` |
| `{{name}}` | Filename without extension | `document` |
| `{{extension}}` | File extension | `pdf` |
| `{{size}}` | File size in bytes | `1024` |
| `{{created_year}}` | Creation year | `2024` |
| `{{created_month}}` | Creation month | `03` |
| `{{created_day}}` | Creation day | `15` |
| `{{modified_year}}` | Modification year | `2024` |
| `{{modified_month}}` | Modification month | `03` |
| `{{modified_day}}` | Modification day | `15` |

---

## ⚡ Performance

Tooka is designed for speed and efficiency:

- **Parallel processing** using Rust's rayon for concurrent file operations
- **Optimized traversal** with priority-sorted rules for early termination
- **Memory efficient** streaming operations for large directories
- **Benchmarked performance** with Criterion.rs in `benches/` directory

Example benchmark results (sorting 1000 files):
- File collection: ~1.2ms
- Rule matching: ~5.5ms
- Total operation: ~12ms

---

## 🛡️ Safety Features

- **Dry-run mode** (`--dry-run`) to preview operations
- **Comprehensive logging** with multiple verbosity levels
- **Validation** of rules before execution
- **Error handling** with detailed error messages
- **Backup recommendations** before bulk operations

---

## 🔨 Installation

### From Source
```bash
git clone https://github.com/tooka-org/tooka.git
cd tooka
cargo build --release
./target/release/tooka --help
```

### From Crates.io
```bash
cargo install tooka
```

### Pre-built Binaries
Download from [GitHub Releases](https://github.com/tooka-org/tooka/releases)

---

## 📚 Documentation

- **Getting Started Guide** - Complete setup and first steps
- **Rule Reference** - Detailed rule syntax and examples  
- **CLI Reference** - All commands and options
- **Template Guide** - Advanced templating features
- **Performance Tips** - Optimization best practices

---

## 🤝 Contributing

We welcome contributions! Please see:

- [Contributing Guidelines](CONTRIBUTING.md) 
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [GitHub Discussions](https://github.com/tooka-org/tooka/discussions) for ideas and questions

---

## 💬 Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/tooka-org/tooka/issues)
- **💡 Feature Requests**: [GitHub Discussions](https://github.com/tooka-org/tooka/discussions)
- **📋 Quick Feedback**: [Feedback Form](https://tally.so/r/mBVyLe) (no account needed)
- **📖 Documentation**: [Wiki](https://github.com/tooka-org/tooka/wiki)

---

## 📜 License

Licensed under [GPLv3](LICENSE)

---

<div align="center">
  <sub>Built with ❤️ in Rust</sub>
</div>
