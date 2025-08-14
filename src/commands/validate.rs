use crate::rules::rule::Rule;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "✅ Validate a rule YAML file against the schema")]
pub struct ValidateArgs {
    /// Path to the rule YAML file
    #[arg(value_name = "FILE", help = "Path to the YAML file to validate")]
    pub file: String,

    /// Optional flag to do a full validation, including value limits
    #[arg(
        long,
        default_value_t = false,
        help = "Perform deep validation including value limits"
    )]
    pub deep: bool,
}

pub fn run(args: &ValidateArgs) -> Result<()> {
    log::info!("Validating rule from file: {}", args.file);

    // Deserialize the file (already validates structure)
    let rules = Rule::new_from_file(&args.file)
        .map_err(|e| anyhow::anyhow!("Failed to load rule from file: {}: {}", args.file, e))?;

    log::info!("Loaded {} rules from file: {}", rules.len(), args.file);
    println!("Loaded {} rules from file: {}", rules.len(), args.file);

    if !args.deep {
        println!("✅ File is structurally valid (schema match)");
        return Ok(());
    }

    // Only validate rule content when deep is true
    let mut err_count = 0;
    for rule in rules {
        if let Err(e) = rule.validate(true) {
            log::error!("Rule '{}' is invalid: {}", rule.name, e);
            println!("❌ Rule '{}' is invalid: {}", rule.name, e);
            err_count += 1;
        } else {
            log::info!("Rule '{}' is valid", rule.name);
            println!("✅ Rule '{}' is valid", rule.name);
        }
    }

    if err_count > 0 {
        log::error!("Validation completed with {err_count} errors");
        println!("Validation completed with {err_count} errors");
        return Err(anyhow::anyhow!(
            "Validation failed with {} errors",
            err_count
        ));
    }

    log::info!("All rules are valid");
    println!("✅ All rules are valid");

    Ok(())
}
