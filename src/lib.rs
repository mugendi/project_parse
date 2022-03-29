#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

//! This module parses a coding project directory and does a few nifty things such as:
//! - Detect the main project language by looking at familiar files such as package.json, Cargo.toml and so on
//! - Generate generic gitignore content based on language(s) detected. Based on these language [gitignores](https://github.com/starship/starship/tree/master/src/configs).
//! - Generate gitignore rules that can then be used to check if any file/directory within the project is ignored
//! - Generate code stats within the project by counting lines oc code LOC for each code file not ignored
//! # How to
//!  
//! ```no_run
//! let dir = "/my/project/dir";
//! // Init new project::Project
//! let mut project = project::Project::new(dir)?;
//! // Parse project
//! project.parse()?;
//! // Add some files to ignore
//! let ignore_str = "files/to/ignore/1.js \n files/to/ignore/2.rs ";
//! // Pass false for update_existing to update generic gitignore. True overwrites generic gitignore
//! project.set_gitignore(ignore_str, &false)?;
//! // If you would like to also add the user defined gitignore
//! project.use_project_gitignore(&true)?;
//! // Check if a specific file is ignored
//! println!("1 {:?}", project.is_ignored("files/to/ignore/1.js"));
//! // Get project code stats. 
//! project.get_code_stats()?;
//! println!("{:#?}", project);
//! ```
//! 


mod code;
mod detector;
mod ruleset;

/// The main project module
pub mod project;

#[cfg(test)]
mod tests {
    use super::project::Project;
    // use crate::project;
    use anyhow::*;
    use std::{env, path::PathBuf};
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum TestError {
        #[error("Test Error")]
        Test,
    }

    fn test_dir(dir: &str) -> String {
        let mut path = env::current_dir().unwrap();
        path.push("test_projects");
        path.push(dir);

        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_detect_rust_lang() -> Result<()> {
        let dir = test_dir("rust");
        let mut project = Project::new(&dir[..])?;
        project.parse()?;
        let result = vec![String::from("rust")];

        assert_eq!(Some(result), project.project_langs);

        Ok(())
    }

    #[test]
    fn test_detect_rust_node() -> Result<()> {
        let dir = test_dir("node");
        let mut project = Project::new(&dir[..])?;
        project.parse()?;
        let result = vec![String::from("node")];

        assert_eq!(Some(result), project.project_langs);

        Ok(())
    }

    #[test]
    fn test_get_gitignore() -> Result<()> {
        let dir = test_dir("node");
        let mut project = Project::new(&dir[..])?;
        project.parse()?;
        let gitignore = project.generic_gitignore.unwrap();
        let gitignore = gitignore[0].as_str();

        // let result = vec![String::from("node")];
        assert_eq!(Some(0), gitignore.find("\n### Node"));
        // assert_eq!(1,3);

        Ok(())
    }

    #[test]
    fn test_get_code_stats() -> Result<()> {
        let dir = test_dir("node");
        let mut project = Project::new(&dir[..])?;
        project.parse()?;

        // get stats
        project.get_code_stats()?;

        assert_eq!(true, project.code_stats.unwrap().contains_key("JSON"));

        Ok(())
    }

    #[test]
    fn test_non_existing_dir() -> Result<()> {
        let dir = "/imagigary/dir";
        let err = match Project::new(&dir[..]) {
            Err(e) => e,
            _ => anyhow!(TestError::Test),
        };

        let err_msg = format!("{:?}", err);
        let expected_err_msg = String::from("Directory /imagigary/dir Cannot be found!");

        assert_eq!(err_msg, expected_err_msg);

        Ok(())
    }
}
