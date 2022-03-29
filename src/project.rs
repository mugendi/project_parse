// Copyright 2022 Anthony Mugendi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{anyhow, Result};
use loc::Count;
use regex::Regex;
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};
use thiserror::Error;

use super::code;
use super::detector;
use super::ruleset;

/// Custom Error for Project
#[derive(Error, Debug)]
pub enum ProjectError {
    /// the NotFound Error occurs when Project is initialized using [method.new] and the string passed points to a directory that doesn't exist
    #[error("Directory {0} Cannot be found!")]
    NotFound(String),
}

/// Project struct 
#[derive(Debug)]
pub struct Project {
    /// project directory path
    pub dir: PathBuf,
    /// option that holds detected project languages
    pub project_langs: Option<Vec<String>>,
    /// option indicating if project directory is also a git directory
    pub is_git: Option<bool>,
    /// option populated with generic git content based on languages detected
    pub generic_gitignore: Option<Vec<String>>,
    /// set of regex rules used to match files & directories to determine if they can be ignored
    pub gitignore_ruleset: Option<ruleset::RuleSet>,
    /// option populated with parsed code statistics for all code files in project directory
    pub code_stats: Option<HashMap<String, loc::Count>>,
}

/// IsIgnored Struct. Returned by the [method.is_ignored] Project implementation
#[derive(Debug)]
pub struct IsIgnored {
    exists: bool,
    is_dir: bool,
    is_ignored: bool,
}


impl Project {
    // create new project
    /// Initializes the Project struct by taking a project directory
    /// ```no_run
    /// let dir = "/my/project/directory";
    /// //Needs to be a mutable variable for other methods to use and update Project
    /// let mut project = project::Project::new(dir)?;
    /// // Adding a file to the generic gitignore
    /// ```
    pub fn new(dir_path: &str) -> Result<Project> {
        let dir_path = PathBuf::from(dir_path);

        // check that dir exists
        if !dir_path.exists() {
            return Err(anyhow!(ProjectError::NotFound(
                dir_path.to_string_lossy().to_string()
            )));
        }
        //init
        let mut project = Project {
            dir: dir_path,
            project_langs: None,

            is_git: None,
            generic_gitignore: None,
            gitignore_ruleset: None,

            code_stats: None,
        };

        project.is_git()?;

        Ok(project)
    }

    /// Parses the Project initialized with [method.new]
    /// Parsing will perform the following key tasks:
    /// - Detect main project language(s) 
    /// - Generate a generic gitignore based on [gitignores](https://github.com/starship/starship/tree/master/src/configs)
    /// - Generate Regexp rules from the generic gitignore that are used to check if files and directories within the project should be git-ignored.
    pub fn parse(&mut self) -> Result<()> {
        // extend via impl methods
        self.add_langs()?;
        self.add_gitignore()?;
        self.get_rules()?;
        Ok(())
    }

    /// Generates code stats for all the project files that are:
    /// - Code files. The following file types are supported
    /// - Not ignored based on the gitignore rules
    pub fn get_code_stats(&mut self) -> Result<Option<HashMap<String, Count>>> {
        // rrr
        let stats = code::dir_stats(&self.dir, &self.gitignore_ruleset)?;

        self.code_stats = stats.clone();

        Ok(stats)
    }

    /// Check if directory or file within the project folder is ignored based on:
    /// - The project generic gitignore (based on )
    /// - Any extra gitignore rules passed via [method.set_gitignore] and [method.use_project_gitignore]
    pub fn is_ignored(&self, path_str: &str) -> Option<IsIgnored> {
        let mut blank_ignored = IsIgnored {
            exists: false,
            is_dir: false,
            is_ignored: false,
        };

        //
        let is_ignored = match &self.gitignore_ruleset {
            Some(ruleset) => {
                // get proper dir
                let path = PathBuf::from(path_str);
                let path = if path.is_relative() {
                    let mut path = self.dir.clone();
                    path.push(path_str);
                    path
                } else {
                    path
                };

                // quick determine based on whether there is a file ext
                let re = Regex::new(r"\.\w{2}$").unwrap();
                let mut is_dir = !re.is_match(path_str);

                // only if path exists...
                if path.exists() {
                    blank_ignored.exists = true;
                    // check if is dir from metadata
                    is_dir = path.metadata().expect("Cannot get metadata").is_dir();
                }

                // update is dir
                blank_ignored.is_dir = is_dir;

                // is it ignored based on the rules?
                blank_ignored.is_ignored = ruleset.is_ignored(path, is_dir);

                blank_ignored
            }
            // no ruleset???
            _ => blank_ignored,
        };

        Some(is_ignored)
    }

    /// Allows you to set your own gitignore rules by passing them as a &str param
    /// You can set update_existing to true to update the generic gitignore from [gitignores](https://github.com/starship/starship/tree/master/src/configs) or false to overwrite it
    /// **Example** 
    /// ```no_run
    /// let ignore_str = "ignore/this/file.js";
    /// project.set_gitignore(ignore_str, &true)?;
    /// ```
    ///     
    pub fn set_gitignore(&mut self, git_str: &str, update_existing: &bool) -> Result<()> {
        //get new or updated ignore text
        let mut ignore_text = match self.generic_gitignore.clone() {
            Some(gitignore) => {
                if *update_existing {
                    gitignore.clone()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        };

        // add git str ensuring we add new line first
        ignore_text.push(format!("\n {}", git_str));

        // println!("{:?}", ignore_text);
        self.generic_gitignore = Some(ignore_text);
        // update rules
        self.get_rules()?;

        Ok(())
    }

    /// Allows one to use the project's own .gitignore file
    /// When ```update_generic``` is true, then the project .gitgnore is merged with a generic gitignore from [gitignores](https://github.com/starship/starship/tree/master/src/configs)
    /// ```no_run
    /// project.use_project_gitignore(&false)?;
    /// ```
    pub fn use_project_gitignore(&mut self, update_generic: &bool) -> Result<()> {
        // read .gitignore
        let mut path = self.dir.clone();
        path.push(".gitignore");

        // if path exists
        let gitignore = if path.exists() {
            // read file
            match read_to_string(path) {
                Ok(s) => s,
                _ => "".into(),
            }
        } else {
            "".into()
        };

        if *update_generic {
            self.set_gitignore(&gitignore[..], &true)?;
        } else {
            self.generic_gitignore = Some(vec![gitignore]);
            // update rules
            self.get_rules()?;
        }
        

        Ok(())
    }
 
    fn get_rules(&mut self) -> Result<()> {
        let dir = &self.dir;
        let empty_ruleset = ruleset::RuleSet::new(&dir, vec![""])?;

        let rule_set: ruleset::RuleSet = match &self.generic_gitignore {
            Some(git_ignores) => {
                // join multiple rules separating them with new lines
                let content = git_ignores.join("\n\n");
                match ruleset::load_str(&dir, &content[..]) {
                    Ok(ruleset) => ruleset,
                    _ => empty_ruleset,
                }
            }
            _ => empty_ruleset,
        };

        self.gitignore_ruleset = Some(rule_set);

        Ok(())
    }

    fn add_langs(&mut self) -> Result<()> {
        // get lang match pattern
        let langs = Some(detector::detect_lang_from_dir(&self.dir)?);

        self.project_langs = langs.clone();

        Ok(())
    }

    fn add_gitignore(&mut self) -> Result<()> {
        // get lang match pattern
        let git_ignores = detector::get_lang_gitignore(&self.project_langs)?;

        self.generic_gitignore = git_ignores.clone();

        Ok(())
    }

    fn is_git(&mut self) -> Result<()> {
        // Check if .git dir exists within project
        let mut dir = self.dir.clone();
        dir.push(".git");

        self.is_git = Some(dir.exists());

        Ok(())
    }
}
