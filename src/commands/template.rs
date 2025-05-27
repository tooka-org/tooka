use crate::core::template::generate_rule_template_yaml;
use clap::Args;

#[derive(Args)]
#[command(about = "Exports a template rule to a YAML file")]
pub struct TemplateArgs {
    /// Output file path
    #[arg(long)]
    pub output: Option<String>,
}

pub fn run(args: TemplateArgs) {
    println!("Generating rule template YAML");
    let output_path = args
        .output
        .unwrap_or_else(|| "rule_template.yaml".to_string());
    log::info!("Generating rule template YAML to {}", output_path);

    match generate_rule_template_yaml() {
        Ok(template_yaml) => {
            if let Err(e) = std::fs::write(&output_path, template_yaml) {
                println!("Error writing to file: {}", e);
                log::error!("Error writing rule template to {}: {}", output_path, e);
            } else {
                println!("Rule template generated successfully!");
                log::info!("Rule template generated successfully at: {}", output_path);
            }
        }
        Err(e) => {
            println!("Error generating rule template: {}", e);
            log::error!("Error generating rule template: {}", e);
        }
    }
}
