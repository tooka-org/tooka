//! # Tooka Core
//!
//! [![github]](https://github.com/Benji377/tooka)&ensp;[![crates-io]](https://crates.io/crates/tooka-core)&ensp;[![docs-rs]](https://docs.rs/tooka-core)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! The internal rule engine for [`tooka`](https://github.com/Benji377/tooka), a powerful CLI for organizing files using declarative YAML rules.
//!
//! ---
//!
//! ## Overview
//!
//! `tooka-core` exposes the core components that enable the Tooka CLI to:
//!
//! - Load and validate YAML-based rule files
//! - Recursively traverse directories in parallel
//! - Apply conditional filters based on metadata, timestamps, and more
//! - Perform actions like move, rename, copy, delete, or skip
//!
//! It's designed for reusability, making it easy to embed file-handling automation into other Rust apps.
//!
//! ---
//!
//! See the full source and CLI wrapper:  
//! [github.com/Benji377/tooka](https://github.com/Benji377/tooka)
//!

mod common;
mod core;
mod file;
mod rules;
mod utils;

pub use common::{config::Config, logger};
pub use core::{context, error, report, sorter};
pub use file::{file_match, file_ops};
pub use rules::{rule, rules_file, template};
