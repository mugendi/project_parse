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
use loc::{Count, Lang};
use std::{collections::HashMap, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::ruleset;
// pub struct S

fn code_stats(e: &DirEntry) -> Result<(Lang, Count)> {
    let path_str = e.path().to_str().unwrap();

    let count: Count;
    let lang = loc::lang_from_ext(path_str);

    if lang != Lang::Unrecognized {
        // count lines
        count = loc::count(path_str);
    } else {
        count = Count {
            code: 0,
            comment: 0,
            blank: 0,
            lines: 0,
        }
    }

    // let lang_str = lang.to_s().clone();
    // let lang_str = lang.to_s();

    Ok((lang, count))
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_file(entry: &DirEntry) -> bool {
    // entry.metadata().expect("Could not get metadata").is_file();
    entry.metadata().expect("Could not get metadata").is_file()
}

pub fn is_ignored(ruleset: &ruleset::RuleSet, entry: &DirEntry) -> bool {
    let e = entry;
    let is_dir = e.metadata().expect("Could not get metadata").is_dir();

    let is_ignored = ruleset.is_ignored(e.path(), is_dir);

    // println!("{:?} -> {:?}", is_ignored, e.path());

    is_ignored
}

pub fn dir_stats(
    dir: &PathBuf,
    ruleset: &Option<ruleset::RuleSet>,
) -> Result<Option<HashMap<String, Count>>> {
    let dir_str = dir.to_str().unwrap();
    let mut stats: HashMap<String, Count> = HashMap::new();
    let walker = WalkDir::new(dir_str).into_iter();

    match ruleset {
        Some(ruleset) => {

            for entry in walker.filter_entry(|e| !is_hidden(e) && !is_ignored(&ruleset, e)) {
                let e = entry?;

                if is_file(&e) {
                    //
                    let (lang, count) = code_stats(&e)?;
                    let lang = lang.clone();
                    let lang_str = lang.to_s().to_string();

                    // println!("\nlang: {} \n count: {:?}", lang_str, count);
                    // stats[]
                    let stat = stats.entry(lang_str).or_insert(Count {
                        code: 0,
                        comment: 0,
                        blank: 0,
                        lines: 0,
                    });

                    stat.merge(&count);

                    // println!(">> {:?}", stat);
                }
            }
        }
        _ => (),
    }

    // println!("{:#?}", stats);
    let stats = if stats.len() > 0 { Some(stats) } else { None };

    Ok(stats)
}
