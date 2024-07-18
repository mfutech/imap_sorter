use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
#[derive(Debug, Deserialize, Serialize, Clone)]
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

        // check if tag if in the tag list
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
            // "{:<25} filter: {:<60} target: {:<15} tags: {:<20}",
            "* rule:\t{}\n\tfilter: {}\n\ttarget: {}\n\ttags: {}",
            &self.name,
            &self.filter,
            &self.target,
            &self.tags_string()
        )
    }

    pub fn print(&self) {
        println!("{}", &self.as_string())
    }

    pub fn name_and_tag(&self) -> String {
        let tags = match &self.tags {
            Some(tags) => format!(" [{}]", tags.join(", ")),
            _ => "".to_string(),
        };
        format!("{}{}", &self.name, tags)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FolderRule {
    pub folder: String,
    pub folders: Option<Vec<String>>,
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
        let mut all_tags: Vec<String> = self
            .folders
            .iter()
            .flat_map(|fld| fld.list_tags())
            .collect();
        all_tags.sort();
        all_tags.dedup();
        all_tags
    }

    pub fn list_folders(&self) -> Vec<String> {
        // extract all folders from config, either in name or in folders parameter
        let all_folders: Vec<String> = self
            .folders
            .iter()
            .flat_map(|folder| {
                let mut folders = vec![folder.folder.clone()];
                if let Some(folder_list) = &folder.folders {
                    folders.extend(folder_list.clone());
                }
                folders
            })
            .collect();

        // reduce list to unique folder, while preserving order of folder as defined in configuration file
        let mut seen = HashSet::new(); // keep track of what has been seen
        let mut deduplicated = Vec::new(); // deduplicate list of folder

        for folder in all_folders {
            if seen.insert(folder.clone()) {
                deduplicated.push(folder.clone());
            }
        }

        deduplicated
    }

    pub fn rules_for_folder(&self, folder: String) -> Vec<Rule> {
        self.folders
            .iter()
            .flat_map(|fld| {
                // for each folder, that match, collect rules
                let mut rules: Vec<Rule> = Vec::new();

                // check if this is the folder name
                if fld.folder == folder {
                    rules.extend_from_slice(&fld.rules);
                };

                // if multiple folders are named in the folders (with a s) paratmer then check them all
                if let Some(folder_list) = &fld.folders {
                    if folder_list.contains(&folder) {
                        // matching, then adding
                        rules.extend_from_slice(&fld.rules)
                    }
                }
                rules
            })
            .collect()
    }

    pub fn print(&self) {
        for folder in &self.folders {
            folder.print();
        }
    }
}
