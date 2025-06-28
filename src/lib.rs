// This lib.rs file exists solely for benchmarking purposes.
// The main application is a CLI binary, not a library.
//
// This library interface only exposes the modules needed for criterion benchmarks
// to access the sort_files function and its dependencies for performance testing.

pub mod common {
    pub mod config;
    pub mod environment;
    pub mod logger;
}

pub mod core {
    pub mod context;
    pub mod error;
    pub mod sorter;
}

pub mod file {
    pub mod file_match;
    pub mod file_ops;
}

pub mod rules {
    pub mod rule;
    pub mod rules_file;
}

pub mod utils {
    pub mod rename_pattern;
}
