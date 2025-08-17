use crate::cli::display;
use crate::core::context;
use anyhow::Result;
use clap::Args;
use std::fs;
use std::path::Path;

#[derive(Args)]
#[command(about = "📝 Add a new rule by importing a YAML file or scanning a directory")]
pub struct AddArgs {
    /// Path to the rule YAML file or directory containing YAML files
    #[arg(
        value_name = "PATH",
        help = "Path to the YAML file or directory containing YAML files with rule definitions"
    )]
    pub path: String,

    /// Optional flag to overwrite existing rules
    #[arg(
        long,
        default_value_t = false,
        help = "Overwrite existing rule if it already exists"
    )]
    pub overwrite: bool,
}

pub fn run(args: &AddArgs) -> Result<()> {
    let path = Path::new(&args.path);

    if path.is_file() {
        // Handle single file
        display::info(&format!("📝 Adding rule from file: {}", args.path));
        log::info!("Adding rule from file: {}", args.path);

        let mut rf = context::get_locked_rules_file()?;

        rf.add_rule_from_file(&args.path, args.overwrite)
            .map_err(|e| anyhow::anyhow!("Failed to add rule from file: {}: {}", args.path, e))?;

        display::success("Rule added successfully!");
        log::info!("Rule added successfully from file: {}", args.path);
    } else if path.is_dir() {
        // Handle directory
        display::info(&format!(
            "📂 Scanning directory for YAML files: {}",
            args.path
        ));
        log::info!("Scanning directory for YAML files: {}", args.path);

        let yaml_files = find_yaml_files(path)?;

        if yaml_files.is_empty() {
            display::warning("No YAML files found in the directory");
            log::warn!("No YAML files found in directory: {}", args.path);
            return Ok(());
        }

        display::info(&format!("Found {} YAML files", yaml_files.len()));
        log::info!(
            "Found {} YAML files in directory: {}",
            yaml_files.len(),
            args.path
        );

        let mut rf = context::get_locked_rules_file()?;
        let mut added_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;

        for file_path in yaml_files {
            let file_path_str = file_path.to_string_lossy();
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            log::info!("Processing file: {file_path_str}");

            match rf.add_rule_from_file(&file_path_str, args.overwrite) {
                Ok(()) => {
                    display::success(&format!("  ✅ Added rules from: {file_name}"));
                    log::info!("Successfully added rules from: {file_path_str}");
                    added_count += 1;
                }
                Err(e) => {
                    if e.to_string().contains("already exists") && !args.overwrite {
                        display::warning(&format!("  ⚠️  Skipped (rule exists): {file_name}"));
                        log::warn!("Skipped file due to existing rule: {file_path_str}");
                        skipped_count += 1;
                    } else {
                        display::error(&format!("  ❌ Failed to add from: {file_name} - {e}"));
                        log::error!("Failed to add rules from: {file_path_str} - {e}");
                        failed_count += 1;
                    }
                }
            }
        }

        // Print summary
        display::info(&format!(
            "📊 Summary: {added_count} added, {skipped_count} skipped, {failed_count} failed"
        ));
        log::info!(
            "Directory processing complete. Added: {added_count}, Skipped: {skipped_count}, Failed: {failed_count}"
        );

        if failed_count > 0 {
            return Err(anyhow::anyhow!("Failed to process {} files", failed_count));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Path is neither a file nor a directory: {}",
            args.path
        ));
    }

    Ok(())
}

/// Find all YAML files in a directory (non-recursive)
fn find_yaml_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut yaml_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    yaml_files.push(path);
                }
            }
        }
    }

    // Sort files for consistent ordering
    yaml_files.sort();
    Ok(yaml_files)
}
