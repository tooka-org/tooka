# 🧩 Tooka Core

[![github]](https://github.com/Benji377/tooka)&ensp;[![crates-io]](https://crates.io/crates/tooka-core)&ensp;[![docs-rs]](https://docs.rs/tooka_core)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

The internal engine powering the Tooka CLI — a rule-based automation framework for file handling.

---

## 🧭 Overview

**Tooka Core** is the logic layer behind the Tooka CLI. It enables robust file traversal, filtering, and automation using declarative YAML rules.

Designed to be decoupled from the CLI, this crate is suitable for building custom file management apps or services.

---

## ✨ Core Features

- YAML-driven rule parsing and execution
- Recursive, parallel file system traversal
- Conditional filtering by name, size, metadata, etc.
- Flexible actions: move, copy, rename, delete, skip
- Custom templating for output filenames
- Dry-run mode for safe testing
- Detailed logging via `flexi_logger`

---

## 🚀 Use Cases

- Build file cleanup utilities
- Automate media organization (images, documents, etc.)
- Create backups or archive flows
- Filter and tag datasets by metadata

---

## 📦 Integration Example

Add `tooka-core` to your `Cargo.toml`:

```toml
[dependencies]
tooka-core = "1.0.0"
```

---

## 📜 License

Licensed under [GPLv3](../LICENSE)