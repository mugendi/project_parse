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
use globset::{Candidate, GlobBuilder, GlobSet, GlobSetBuilder};
use std::fmt;
use std::path::{Path, PathBuf};

/// Represents a set of rules that can be checked against to see if a path should be ignored within
/// a Git repository.
///
/// The performance characteristics of this are such that it is much better to try and make a single
/// instance of this to check as many paths against as possible - this is because the highest cost
/// is in constructing it, but checking against the compiled patterns is extremely cheap.
///
// #[derive(Copy)]

// #[derive(Debug)]
pub struct RuleSet {
    root: PathBuf,
    pub(crate) rules: Vec<Rule>,
    tester: GlobSet,
}

impl RuleSet {
    /// Construct a ruleset, given a path that is the root of the repository, and a set of rules,
    /// which is a vector
    pub fn new(root: &PathBuf, raw_rules: Vec<&str>) -> Result<RuleSet> {
        // FIXME: Is there a better way without needing to hardcode a path here?

        let cleaned_root = Self::strip_prefix(root, Path::new("./"));

        let lines = raw_rules
            .into_iter()
            .map(RuleSet::parse_line)
            .collect::<Result<Vec<ParsedLine>>>()?;

        let rules: Vec<Rule> = lines
            .iter()
            .filter_map(|parsed_line| {
                match parsed_line {
                    // FIXME: Remove this clone if possible, it's rank.
                    &ParsedLine::WithRule(ref rule) => Some(rule.clone()),
                    _ => None,
                }
            })
            .collect();

        let mut tester_builder = GlobSetBuilder::new();

        // Add globs to globset.
        for rule in rules.iter() {
            let mut glob_builder = GlobBuilder::new(&rule.pattern);
            glob_builder.literal_separator(rule.anchored);
            let glob = glob_builder.build()?;
            tester_builder.add(glob);
        }

        let tester = tester_builder.build()?;

        Ok(RuleSet {
            root: cleaned_root,
            rules,
            tester,
        })
    }

    /// Check if the given path should be considered ignored as per the rules contained within
    /// the current ruleset.
    pub fn is_ignored<P: AsRef<Path>>(&self, path: P, is_dir: bool) -> bool {
        // FIXME: Is there a better way without needing to hardcode a path here?
        let mut cleaned_path = Self::strip_prefix(path.as_ref(), Path::new("./"));
        cleaned_path = Self::strip_prefix(cleaned_path.as_path(), &self.root);

        let candidate = Candidate::new(&cleaned_path);
        let results = self.tester.matches_candidate(&candidate);

        for idx in results.iter().rev() {
            let ref rule = self.rules[*idx];

            // We must backtrack through the finds until we find one that is_dir
            // and rule.dir_only agree on.
            if rule.dir_only && !is_dir {
                continue;
            }

            return !rule.negation;
        }

        false
    }

    /// Given a raw pattern, parse it and attempt to construct a rule out of it. The pattern pattern
    /// rules are implemented as described in the documentation for Git at
    /// https://git-scm.com/docs/gitignore.
    fn parse_line<R: AsRef<str>>(raw_rule: R) -> Result<ParsedLine> {
        // FIXME: Can we combine some of these string scans?
        let mut pattern = raw_rule.as_ref().trim();

        if pattern.is_empty() {
            return Ok(ParsedLine::Empty);
        }

        if pattern.starts_with('#') {
            return Ok(ParsedLine::Comment);
        }

        let negation = pattern.starts_with('!');
        if negation {
            pattern = pattern.trim_start_matches('!').trim();
        }

        let dir_only = pattern.ends_with('/');
        if dir_only {
            pattern = pattern.trim_end_matches('/').trim();
        }

        let absolute = pattern.starts_with('/');
        if absolute {
            pattern = pattern.trim_start_matches('/');
        }

        let anchored = absolute || pattern.contains('/');

        let mut cleaned_pattern = if !absolute && !pattern.starts_with("**/") {
            format!("**/{}", pattern.replace(r"\", ""))
        } else {
            pattern.replace(r"\", "")
        };

        // If the glob ends with `/**`, then we should only match everything
        // inside a directory, but not the directory itself. Standard globs
        // will match the directory. So we add `/*` to force the issue.
        if cleaned_pattern.ends_with("/**") {
            cleaned_pattern = format!("{}/*", cleaned_pattern);
        }

        Ok(ParsedLine::WithRule(Rule {
            pattern: cleaned_pattern, // FIXME: This is not zero-copy.
            anchored,
            dir_only,
            negation,
        }))
    }

    /// Given a path and a prefix, strip the prefix off the path. If the path does not begin with
    /// the given prefix, then return the path as is.
    fn strip_prefix<P: AsRef<Path>, PR: AsRef<Path>>(path: P, prefix: PR) -> PathBuf {
        path.as_ref()
            .strip_prefix(prefix.as_ref())
            .unwrap_or(path.as_ref())
            .to_path_buf()
    }
}

impl fmt::Debug for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} gitignore RULES", self.rules.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Rule {
    pub pattern: String,
    /// Whether this rule is anchored. If a rule is anchored (contains a slash)
    /// then wildcards inside the rule are not allowed to match a `/` in the
    /// pathname.
    pub anchored: bool,
    /// Whether this rule is only allowed to match directories.
    pub dir_only: bool,
    /// Whether the rule should, if it matches, negate any previously matching
    /// patterns. This flag has no effect if no previous patterns had matched.
    pub negation: bool,
}

enum ParsedLine {
    Empty,
    Comment,
    WithRule(Rule),
}

pub fn load_str(root: &PathBuf, content: &str) -> Result<RuleSet> {
    //
    let split = content.split("\n");
    let lines = split.collect::<Vec<&str>>();

    let rule_set = RuleSet::new(root, lines.clone())?;

    // println!(" {:?}", lines);

    Ok(rule_set)
}
