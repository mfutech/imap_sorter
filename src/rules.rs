use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fmt::format;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    pub filter: String,
    pub target: String,
    pub enable: bool,
    pub tags: Option<Vec<String>>,
}

impl Rule {
    pub fn match_tag(&self, tag: &Option<String>) -> bool {
        let tag = match tag {
            Some(tag) => tag.clone(),
            None => return true, // if no tag requested, then it matches
        };

        // from here we know we need to match a tag
        let rule_tags = match &self.tags {
            Some(tags) => tags,
            _ => return false, // if rules has no tags, then cannot match the request tag
        };
        rule_tags.contains(&tag)
    }
    pub fn tags_string(&self) -> String {
        match &self.tags {
            Some(tags) => tags.join(", "),
            _ => String::from(""),
        }
    }

    pub fn as_string(&self) -> String {
        format!(
            "{:<20} filter: {}, target: {}",
            &self.name, &self.filter, &self.target,
        )
    }

    pub fn print(&self) {
        println!("{}", &self.as_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FolderRule {
    pub folder: String,
    pub rules: Vec<Rule>,
}

impl FolderRule {
    pub fn list_tags(&self) -> Vec<String> {
        let mut all_tags: Vec<String> = Vec::new();
        for rule in &self.rules {
            if let Some(tag) = &rule.tags {
                all_tags.extend_from_slice(&tag)
            }
        }
        all_tags.sort();
        all_tags.dedup();
        all_tags
    }

    pub fn print(&self) {
        println!("Folder: {}", &self.folder);
        for rule in &self.rules {
            rule.print();
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RulesSet {
    pub folders: Vec<FolderRule>,
}

impl RulesSet {
    pub fn load(file_name: &str) -> Result<Self> {
        let file =
            File::open(file_name).with_context(|| format!("Failed to open file: {}", file_name))?;
        let reader = BufReader::new(file);
        let rules_set: RulesSet = serde_yaml::from_reader(reader)
            .with_context(|| format!("Failed to parse YAML file: {}", file_name))?;
        Ok(rules_set)
    }
    pub fn list_tags(&self) -> Vec<String> {
        let mut all_tags: Vec<String> = Vec::new();
        for folder in &self.folders {
            all_tags.extend_from_slice(&folder.list_tags())
        }
        all_tags.sort();
        all_tags.dedup();
        all_tags
    }

    pub fn print(&self) {
        for folder in &self.folders {
            folder.print();
        }
    }
}
