use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    pub filter: String,
    pub target: String,
    pub enable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rules {
    pub rules: Vec<Rule>,
}

impl Rules {
    pub fn load(file_name: &str) -> Result<Self> {
        let file =
            File::open(file_name).with_context(|| format!("Failed to open file: {}", file_name))?;
        let reader = BufReader::new(file);
        let rules: Rules = serde_yaml::from_reader(reader)
            .with_context(|| format!("Failed to parse YAML file: {}", file_name))?;
        Ok(rules)
    }
}
