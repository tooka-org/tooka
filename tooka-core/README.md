# 🧩 Tooka Core

[![github]](https://github.com/Benji377/tooka)&ensp;[![crates-io]](https://crates.io/crates/tooka-core)&ensp;[![docs-rs]](https://docs.rs/tooka_core)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

The internal engine powering the Tooka CLI — a rule-based automation framework for file handling.

---

## 🧭 Overview

`tooka-core` provides:

* Declarative YAML rule parsing and validation
* Parallel recursive file traversal
* Conditional filtering on filenames, size, metadata, and timestamps
* Actions including move, copy, rename, delete, and skip
* Template support for dynamic file naming
* Dry-run support and detailed logging

Designed for embedding in CLI tools or other Rust applications.

---

## 🚀 Use Cases

* Automated file cleanup
* Media and document organization
* Backup and archival workflows
* Metadata-driven dataset filtering

---

## 🛠 Integration

Add to your `Cargo.toml`:

```toml
[dependencies]
tooka-core = "1.0.1"
```

---

## ⚡ Performance Benchmarks

Benchmarks using [Criterion.rs](https://github.com/bheisler/criterion.rs) are located in the `benches/` directory to track and optimize sorting and traversal performance.

---

## 📜 License

Licensed under [GPLv3](../LICENSE)
