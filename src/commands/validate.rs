use anyhow::Result;
use clap::Args;
use tooka_core::rule::Rule;

#[derive(Args)]
#[command(about = "validates a rule YAML file against the schema")]
pub struct ValidateArgs {
    /// Path to the rule YAML file
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Optional flag to do a full validation, including value limits
    #[arg(long, default_value_t = false)]
    pub deep: bool,
}

pub fn run(args: &ValidateArgs) -> Result<()> {
    log::info!("Validating rule from file: {}", args.file);
    let mut err_count = 0;

    let rules = Rule::new_from_file(&args.file)
        .map_err(|e| anyhow::anyhow!("Failed to load rule from file: {}: {}", args.file, e))?;

    log::info!("Loaded {} rules from file: {}", rules.len(), args.file);
    println!("Loaded {} rules from file: {}", rules.len(), args.file);

    for rule in rules {
        if let Err(e) = rule.validate(args.deep) {
            log::error!("Rule '{}' is invalid: {}", rule.name, e);
            println!("Rule '{}' is invalid: {}", rule.name, e);
            err_count += 1;
        } else {
            log::info!("Rule '{}' is valid", rule.name);
            println!("Rule '{}' is valid", rule.name);
        }
    }

    if err_count > 0 {
        log::error!("Validation completed with {} errors", err_count);
        println!("Validation completed with {} errors", err_count);
        return Err(anyhow::anyhow!(
            "Validation failed with {} errors",
            err_count
        ));
    }
    log::info!("All rules are valid");
    println!("All rules are valid");

    Ok(())
}
