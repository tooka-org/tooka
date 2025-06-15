# 🗂️ Tooka

[![crates-io]](https://crates.io/crates/tooka-core)
[![docs-rs]](https://docs.rs/tooka_core)
[![clippy]](https://github.com/Benji377/tooka/actions/workflows/clippy.yml)
[![test]](https://github.com/Benji377/tooka/actions/workflows/test.yml)
[![feedback]](https://tally.so/r/mBVyLe)

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

📚 Full usage examples and explanations in the [Wiki](https://github.com/Benji377/tooka/wiki).

---

## 🌐 Website

Visit [**tooka.deno.dev**](https://tooka.deno.dev) for:

* 📦 **Downloads** – prebuilt binaries for macOS, Windows, and Linux
* 🛠️ **Rule Builder** – create rules visually and export as YAML

---

## 📚 Wiki

The [**Tooka Wiki**](https://github.com/Benji377/tooka/wiki) covers:

* Installation & configuration
* Rule structure & templates
* CLI commands
* Troubleshooting
* Docker sandbox usage

---

## 💬 Community

Join the [**GitHub Discussions**](https://github.com/Benji377/tooka/discussions) for:

* Feature ideas
* Help and usage tips
* Community showcase

---

## 📝 Feedback

Got thoughts, bugs, or praise?

**👉 [Submit feedback via this form](https://tally.so/r/mBVyLe)** – no GitHub account needed.

[clippy]: https://img.shields.io/github/actions/workflow/status/Benji377/tooka/clippy.yml?label=Clippy&logo=rust&style=for-the-badge&labelColor=555555
[test]: https://img.shields.io/github/actions/workflow/status/Benji377/tooka/test.yml?label=Tests&logo=githubactions&style=for-the-badge&labelColor=555555
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[feedback]: https://img.shields.io/badge/feedback-Tally-blueviolet?style=for-the-badge&labelColor=555555&logo=googleforms

---

## 🌟 Stargazers

[![Stargazers repo roster for @Benji377/tooka](https://reporoster.com/stars/dark/Benji377/tooka)](https://github.com/Benji377/tooka/stargazers)
[![Forkers repo roster for @Benji377/tooka](https://reporoster.com/forks/dark/Benji377/tooka)](https://github.com/Benji377/tooka/network/members)
## Star History

<a href="https://www.star-history.com/#Benji377/tooka&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=Benji377/tooka&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=Benji377/tooka&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=Benji377/tooka&type=Date" />
 </picture>
</a>
