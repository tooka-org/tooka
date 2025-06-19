# 🗂️ Tooka

[![clippy]](https://github.com/tooka-org/cli/actions/workflows/clippy.yml)
[![feedback]](https://tally.so/r/mBVyLe)

[clippy]: https://img.shields.io/github/actions/workflow/status/Benji377/tooka/clippy.yml?label=Clippy&logo=rust&style=for-the-badge&labelColor=555555
[feedback]: https://img.shields.io/badge/feedback-Tally-blueviolet?style=for-the-badge&labelColor=555555&logo=googleforms

<div align="center">
  <img src="./assets/logo-banner.png" alt="Tooka logo" style="width: 80%">
</div>

A fast, rule-based CLI tool for organizing files.

---

## 🧭 Introduction

**Tooka** is a flexible command-line tool for automating your filesystem: organize, rename, move, copy, or delete files using simple, powerful YAML rules.

You define what files to match (by name, extension, metadata, size, etc.) and what should happen to them — Tooka handles the rest.

---

## 🚀 Quick Start

1. **Download** Tooka from the [📦 Website](https://tooka.deno.dev)
2. **Verify install**:

   ```bash
   tooka --version
   ```

3. **Create a rule** using the [🛠️ Rule Builder](https://tooka.deno.dev/builder) or manually with YAML

4. **Run a dry sort**:

   ```bash
   tooka sort --dry-run ~/Downloads
   ```

5. **Apply the rule**:

   ```bash
   tooka sort
   ```

📚 Full usage examples and explanations in the [Wiki](https://github.com/tooka-org/cli/wiki).

---

## 🌐 Website

Visit [**tooka.deno.dev**](https://tooka.deno.dev) for:

* 📦 **Downloads** – prebuilt binaries for macOS, Windows, and Linux
* 🛠️ **Rule Builder** – create rules visually and export as YAML

---

## 📚 Wiki

The [**Tooka Wiki**](https://github.com/tooka-org/cli/wiki) covers:

* Installation & configuration
* Rule structure & templates
* CLI commands
* Troubleshooting
* Docker sandbox usage

---

## 💬 Community

Join the [**GitHub Discussions**](https://github.com/orgs/tooka-org/discussions) for:

* Feature ideas
* Help and usage tips
* Community showcase

---

## 📝 Feedback

Got thoughts, bugs, or praise?

**👉 [Submit feedback via this form](https://tally.so/r/mBVyLe)** – no GitHub account needed.
