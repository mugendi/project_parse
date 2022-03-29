# How ?

This module parses a coding project directory and does a few nifty things such as:

- Detect the main project language by looking at familiar files such as package.json, Cargo.toml and so on.
- Generate generic gitignore content based on language(s) detected. Based on these language [gitignores](https://github.com/starship/starship/tree/master/src/configs).
- Generate gitignore rules that can then be used to check if any file/directory within the project is ignored.
- Generate code stats within the project by counting lines oc code LOC for each code file not ignored.

# How to
 
```rust

let dir = "/my/project/dir";
// Init new project::Project
let mut project = project::Project::new(dir)?;

// Parse project
project.parse()?;

// Add some files to ignore
let ignore_str = "files/to/ignore/1.js \n files/to/ignore/2.rs ";

// Pass false for update_existing to update generic 
project.set_gitignore(ignore_str, &false)?;

// If you would like to also add the user defined 
project.use_project_gitignore(&true)?;

// Check if a specific file is ignored
println!("1 {:?}", project.is_ignored("files/to/ignore/1.js"));

// Get project code stats. 
project.get_code_stats()?;
println!("{:#?}", project);

```
