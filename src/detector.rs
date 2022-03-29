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

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs::{read_to_string, write},
    path::PathBuf,
    sync::Mutex,
};
use wax::Glob;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configs {
    pub project_file_types: Vec<String>,
    pub git_ignores: HashMap<String, Language>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Language {
    key: String,
    name: String,
    #[serde(rename = "fileName")]
    file_name: String,
    pub contents: String,
}

pub static CONFIGS: Lazy<Mutex<Configs>> = Lazy::new(|| {
    let project_types = vec![
        "shard.yml",
        "pubspec.yaml",
        "pubspec.yml",
        "pubspec.lock",
        "mix.exs",
        "elm.json",
        "elm-package.json",
        ".elm-version",
        "rebar.config",
        "erlang.mk",
        "stack.yaml",
        "Setup.hs",
        "go.mod",
        "go.sum",
        "glide.yaml",
        "Gopkg.yml",
        "Gopkg.lock",
        ".go-version",
        "build.gradle",
        "pom.xml",
        "build.gradle.kts",
        "build.sbt",
        ".java.version",
        "deps.edn",
        "project.clj",
        "build.boot",
        "Project.toml",
        "Manifest.toml",
        "nim.cfg",
        "package.json",
        ".node-version",
        ".nvmrc",
        "dune",
        "dune-project",
        "jbuild",
        "jbuild-ignore",
        ".merlin",
        "Makefile.PL",
        "Build.PL",
        "cpanfile",
        "cpanfile.snapshot",
        "META.json",
        "META.yml",
        ".perl-version",
        "composer.json",
        ".php-version",
        "spago.dhall",
        "packages.dhall",
        "requirements.txt",
        ".python-version",
        "pyproject.toml",
        "Pipfile",
        "tox.ini",
        "setup.py",
        "__init__.py",
        ".Rprofile",
        "Gemfile",
        ".ruby-version",
        "Cargo.toml",
        ".scalaenv",
        ".sbtenv",
        "build.sbt",
        "Package.swift",
    ];

    let ignores_obj: HashMap<String, Language> = get_ignores().unwrap();

    let configs = Configs {
        project_file_types: project_types
            .iter()
            .map(|e| {
                let derefd = *e;
                derefd.into()
            })
            .collect(),
        git_ignores: ignores_obj,
    };

    Mutex::new(configs)
});

#[derive(Debug)]
pub struct Detectors {
    detectors: Vec<Detector>,
}

impl Detectors {
    pub fn detects<E: DirEntry>(&self, entries: &[E]) -> Vec<String> {
        self.detectors
            .iter()
            .filter_map(|detector| detector.detects(entries))
            .collect()
    }
}

impl Default for Detectors {
    /// Based on https://github.com/starship/starship/tree/master/src/configs
    fn default() -> Self {
        let detectors = vec![
            Detector::new("crystal", [Matcher::by_file_name("shard.yml")]),
            Detector::new(
                "dart",
                [
                    Matcher::by_file_name("pubspec.yaml"),
                    Matcher::by_file_name("pubspec.yml"),
                    Matcher::by_file_name("pubspec.lock"),
                ],
            ),
            Detector::new("elixir", [Matcher::by_file_name("mix.exs")]),
            Detector::new(
                "elm",
                [
                    Matcher::by_file_name("elm.json"),
                    Matcher::by_file_name("elm-package.json"),
                    Matcher::by_file_name(".elm-version"),
                ],
            ),
            Detector::new(
                "erlang",
                [
                    Matcher::by_file_name("rebar.config"),
                    Matcher::by_file_name("erlang.mk"),
                ],
            ),
            Detector::new(
                "haskell",
                [
                    Matcher::by_file_extension("cabal"),
                    Matcher::by_file_name("stack.yaml"),
                    Matcher::by_file_name("Setup.hs"),
                ],
            ),
            Detector::new(
                "go",
                [
                    Matcher::by_file_name("go.mod"),
                    Matcher::by_file_name("go.sum"),
                    Matcher::by_file_name("glide.yaml"),
                    Matcher::by_file_name("Gopkg.yml"),
                    Matcher::by_file_name("Gopkg.lock"),
                    Matcher::by_file_name(".go-version"),
                ],
            ),
            Detector::new(
                "java",
                [
                    Matcher::by_file_name("build.gradle"),
                    Matcher::by_file_name("pom.xml"),
                    Matcher::by_file_name("build.gradle.kts"),
                    Matcher::by_file_name("build.sbt"),
                    Matcher::by_file_name(".java.version"),
                    Matcher::by_file_name("deps.edn"),
                    Matcher::by_file_name("project.clj"),
                    Matcher::by_file_name("build.boot"),
                ],
            ),
            Detector::new(
                "julia",
                [
                    Matcher::by_file_name("Project.toml"),
                    Matcher::by_file_name("Manifest.toml"),
                ],
            ),
            Detector::new("nim", [Matcher::by_file_name("nim.cfg")]),
            Detector::new(
                "node",
                [
                    Matcher::by_file_name("package.json"),
                    Matcher::by_file_name(".node-version"),
                    Matcher::by_file_name(".nvmrc"),
                ],
            ),
            Detector::new(
                "ocaml",
                [
                    Matcher::by_file_name("dune"),
                    Matcher::by_file_name("dune-project"),
                    Matcher::by_file_name("jbuild"),
                    Matcher::by_file_name("jbuild-ignore"),
                    Matcher::by_file_name(".merlin"),
                    Matcher::by_file_extension("opam"),
                ],
            ),
            Detector::new(
                "perl",
                [
                    Matcher::by_file_name("Makefile.PL"),
                    Matcher::by_file_name("Build.PL"),
                    Matcher::by_file_name("cpanfile"),
                    Matcher::by_file_name("cpanfile.snapshot"),
                    Matcher::by_file_name("META.json"),
                    Matcher::by_file_name("META.yml"),
                    Matcher::by_file_name(".perl-version"),
                ],
            ),
            Detector::new(
                "composer", // php
                [
                    Matcher::by_file_name("composer.json"),
                    Matcher::by_file_name(".php-version"),
                ],
            ),
            Detector::new(
                "purescript",
                [
                    Matcher::by_file_name("spago.dhall"),
                    Matcher::by_file_name("packages.dhall"),
                ],
            ),
            Detector::new(
                "python",
                [
                    Matcher::by_file_name("requirements.txt"),
                    Matcher::by_file_name(".python-version"),
                    Matcher::by_file_name("pyproject.toml"),
                    Matcher::by_file_name("Pipfile"),
                    Matcher::by_file_name("tox.ini"),
                    Matcher::by_file_name("setup.py"),
                    Matcher::by_file_name("__init__.py"),
                ],
            ),
            Detector::new("r", [Matcher::by_file_name(".Rprofile")]),
            Detector::new(
                "ruby",
                [
                    Matcher::by_file_extension("gemspec"),
                    Matcher::by_file_name("Gemfile"),
                    Matcher::by_file_name(".ruby-version"),
                ],
            ),
            Detector::new("rust", [Matcher::by_file_name("Cargo.toml")]),
            Detector::new(
                "scala",
                [
                    Matcher::by_file_name(".scalaenv"),
                    Matcher::by_file_name(".sbtenv"),
                    Matcher::by_file_name("build.sbt"),
                ],
            ),
            Detector::new("swift", [Matcher::by_file_name("Package.swift")]),
            Detector::new("zig", [Matcher::by_file_extension("zig")]),
        ];
        Detectors { detectors }
    }
}

#[derive(Debug)]
struct Detector {
    template: String,
    matchers: Vec<Matcher>,
}

impl Detector {
    fn new<T: Into<String>, MS: Into<Vec<Matcher>>>(template: T, matchers: MS) -> Self {
        Detector {
            template: template.into(),
            matchers: matchers.into(),
        }
    }

    fn detects<E: DirEntry>(&self, entries: &[E]) -> Option<String> {
        let result = self
            .matchers
            .iter()
            .any(|matcher| entries.iter().any(|entry| matcher.matches(entry)));
        if result {
            Some(self.template.clone())
        } else {
            None
        }
    }
}

pub trait DirEntry {
    fn file_name(&self) -> OsString;
    fn extension(&self) -> Option<OsString>;
    fn is_file(&self) -> bool;
}

impl DirEntry for std::fs::DirEntry {
    fn file_name(&self) -> OsString {
        self.file_name()
    }

    fn extension(&self) -> Option<OsString> {
        let path = self.path();
        path.extension().map(OsString::from)
    }

    fn is_file(&self) -> bool {
        let path = self.path();
        path.is_file()
    }
}

#[derive(Debug)]
enum Matcher {
    ByFileExtension(OsString),
    ByFileName(OsString),
}

impl Matcher {
    fn by_file_extension<T: Into<OsString>>(extension: T) -> Self {
        Self::ByFileExtension(extension.into())
    }

    fn by_file_name<T: Into<OsString>>(name: T) -> Self {
        Self::ByFileName(name.into())
    }

    fn matches<E: DirEntry>(&self, entry: &E) -> bool {
        match self {
            Self::ByFileName(name) => entry.is_file() && &entry.file_name() == name,
            Self::ByFileExtension(extension) => {
                entry.is_file() && entry.extension() == Some(extension.clone())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FakeDirEntry {
    file_name: OsString,
    extension: Option<OsString>,
    is_file: bool,
}

impl FakeDirEntry {
    fn new<T: Into<OsString>>(file_name: T, extension: Option<T>, is_file: bool) -> Self {
        FakeDirEntry {
            file_name: file_name.into(),
            extension: extension.map(|pe| pe.into()),
            is_file,
        }
    }
}

impl DirEntry for FakeDirEntry {
    fn file_name(&self) -> OsString {
        self.file_name.clone()
    }

    fn extension(&self) -> Option<OsString> {
        self.extension.clone()
    }

    fn is_file(&self) -> bool {
        self.is_file
    }
}

fn get_ignores() -> Result<HashMap<String, Language>> {
    let mut ignores_file = env::temp_dir();
    ignores_file.push("git-ignores.json");

    if ignores_file.exists() {
        // read
        let ignores_str: String = read_to_string(&ignores_file)?;
        let ignores_obj: HashMap<String, Language> =
            serde_json::from_str(&ignores_str).expect("Unable To Parse GitIgnore");
        Ok(ignores_obj)
    } else {
        let git_ignore_url = "https://www.gitignore.io/api/list?format=json";
        let ignores_str: String = ureq::get(git_ignore_url).call()?.into_string()?;
        // save
        write(&ignores_file, &ignores_str)?;
        let ignores_obj: HashMap<String, Language> =
            serde_json::from_str(&ignores_str).expect("Unable To Parse GitIgnore");
        Ok(ignores_obj)
    }
}

pub fn detect_lang(file_path: &PathBuf) -> Result<Vec<String>> {
    let file_name = file_path.file_name().unwrap();
    let ext = file_path.extension();

    let entry = FakeDirEntry::new(file_name, ext, true);
    let result = Detectors::default().detects(&Vec::from([entry]));

    Ok(result)
}

pub fn detect_lang_from_dir(dir: &PathBuf) -> Result<Vec<String>> {
    //
    let mut langs: Vec<String> = Vec::new();
    if dir.metadata().unwrap().is_dir() {
        let configs = CONFIGS.lock().unwrap();

        let types = &configs.project_file_types.join(",");
        let dir_str = dir.to_str().unwrap();

        let dir_str_trimmed = if &dir_str[dir_str.len() - 1..] == "/" {
            &dir_str[..dir_str.len() - 1]
        } else {
            &dir_str
        };

        let glob_search_str = format!("{}/{{*.{}}}", dir_str_trimmed, types);
        //get any of the files used in detection
        let glob = Glob::new(&glob_search_str[..]).unwrap();

        // println!("{:?}", glob_search_str);
        for entry in glob.walk("doc", usize::MAX) {
            // pass entry path
            let matched_file = entry.unwrap().path().to_path_buf();
            // get detected langs & concat
            let entry_langs = detect_lang(&matched_file).unwrap();
            langs = [langs, entry_langs].concat();
        }

        //Langs
        // println!(">>{:?}",  langs);
    }

    Ok(langs)
}

pub fn get_lang_gitignore(langs: &Option<Vec<String>>) -> Result<Option<Vec<String>>> {
    let configs = CONFIGS.lock().unwrap();

    let mut git_ignores: Vec<String> = vec![];

    match langs {
        Some(langs) => {
            // ;
            for lang in langs{
                // println!("LANG {:?}", lang);
                match configs.git_ignores.get(lang){
                    Some(git_ignore)=>{
                        let ignore = git_ignore.contents.clone();
                        git_ignores.push(ignore);
                    },
                    _=>()
                }
            }

            // configs.git_ignores.get("node")
            // Some(String::from(&ignore.contents)),
        }
        _ => (),
    };

    // println!("{:#?}", if git_ignores.len()>0 {Some(git_ignores)} else{None});

    Ok(if git_ignores.len()>0 {Some(git_ignores)} else{None})
}
