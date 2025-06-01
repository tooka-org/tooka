use crate::core::rule::{Action, Match, MetadataMatch, Rule};
use crate::error::TookaError;
use serde_yaml;

pub fn generate_rule_template_yaml() -> Result<String, TookaError> {
    let rule = Rule {
        id: "example_rule".to_string(),
        name: "Example Rule".to_string(),
        enabled: true,
        description: None,
        match_all: false,
        matches: vec![Match {
            extensions: None,
            mime_type: None,
            pattern: None,
            metadata: Some(MetadataMatch {
                exif_date: false,
                fields: vec![],
            }),
            older_than_days: None,
            size_greater_than_kb: None,
            created_between: None,
            filename_regex: None,
            is_symlink: None,
            owner: None,
        }],
        actions: vec![Action::Move {
            destination: "/path/to/destination".to_string(),
            path_template: None,
            create_dirs: false,
        }],
    };

    Ok(serde_yaml::to_string(&rule)?)
}
