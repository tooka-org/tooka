use crate::{
    core::error::TookaError,
    rules::rule::{Action, Conditions, DateRange, MetadataField, MoveAction, Range, Rule},
};

use serde_yaml;

/// Generates a YAML template for a Tooka rule.
pub fn generate_rule_template_yaml() -> Result<String, TookaError> {
    let rule = Rule {
        id: "example_rule".to_string(),
        name: "Example Rule".to_string(),
        enabled: true,
        description: Some("Describe what this rule does".to_string()),
        priority: 1,
        when: Conditions {
            any: Some(false),
            filename: Some(r"^.*\.jpg$".to_string()),
            extensions: Some(vec!["jpg".to_string(), "jpeg".to_string()]),
            path: None,
            size_kb: Some(Range {
                min: Some(10),
                max: Some(5000),
            }),
            mime_type: Some("image/jpeg".to_string()),
            created_date: Some(DateRange {
                from: None,
                to: None,
            }),
            modified_date: None,
            is_symlink: None,
            metadata: Some(vec![MetadataField {
                key: "EXIF:DateTime".to_string(),
                value: None,
            }]),
        },
        then: vec![Action::Move(MoveAction {
            to: "/path/to/destination".to_string(),
            preserve_structure: false,
        })],
    };

    Ok(serde_yaml::to_string(&rule)?)
}
