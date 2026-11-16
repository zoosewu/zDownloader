//! User categories: tags a task gets from rules when it is added (and again once a site module
//! names the file, unless the user changed the task's tags by hand).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CoreError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// Stable id, stored in `Task::tags`.
    pub id: String,
    pub name: String,
    /// Whether every rule must match, or one is enough. A category without rules is assigned
    /// by hand only.
    #[serde(rename = "match")]
    #[ts(rename = "match")]
    pub matching: Matching,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Matching {
    All,
    Any,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Rule {
    /// The file name ends in one of these extensions (`mp4`, `.MKV`; case ignored).
    Extension { values: Vec<String> },
    /// The URL's host is one of these or a subdomain of one (`k2s.cc`).
    Host { values: Vec<String> },
    /// The URL or file name contains `text` (case ignored).
    Contains { target: Target, text: String },
    /// The URL or file name matches `pattern` (Rust regex syntax).
    Regex { target: Target, pattern: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Url,
    Filename,
}

/// Compiled categories, ready to tag tasks.
#[derive(Debug, Clone, Default)]
pub struct Classifier {
    categories: Vec<(String, Matching, Vec<Compiled>)>,
}

#[derive(Debug, Clone)]
enum Compiled {
    Extension(Vec<String>),
    Host(Vec<String>),
    Contains(Target, String),
    Regex(Target, regex::Regex),
}

const MAX_PATTERN: usize = 1000;

impl Classifier {
    /// Checks the categories; errors say what to fix.
    pub fn new(categories: &[Category]) -> Result<Self, CoreError> {
        let mut seen = std::collections::HashSet::new();
        let mut compiled = Vec::new();
        for c in categories {
            let invalid = |why: String| CoreError::Invalid(format!("category {:?}: {why}", c.name));
            if c.id.is_empty() || !seen.insert(c.id.as_str()) {
                return Err(invalid("missing or duplicate id".into()));
            }
            if c.name.trim().is_empty() {
                return Err(CoreError::Invalid("a category needs a name".into()));
            }
            let rules = c.rules.iter().map(|r| compile(r).map_err(&invalid)).collect::<Result<_, _>>()?;
            compiled.push((c.id.clone(), c.matching, rules));
        }
        Ok(Self { categories: compiled })
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.categories.iter().map(|(id, ..)| id.as_str())
    }

    /// The categories a download of `url` saved as `name` belongs to.
    pub fn tags(&self, url: &str, name: &str) -> Vec<String> {
        let subject = Subject::new(url, name);
        self.categories
            .iter()
            .filter(|(_, matching, rules)| {
                !rules.is_empty()
                    && match matching {
                        Matching::All => rules.iter().all(|r| r.matches(&subject)),
                        Matching::Any => rules.iter().any(|r| r.matches(&subject)),
                    }
            })
            .map(|(id, ..)| id.clone())
            .collect()
    }
}

fn compile(rule: &Rule) -> Result<Compiled, String> {
    let list = |values: &[String], trim: &[char]| -> Result<Vec<String>, String> {
        let values: Vec<String> = values
            .iter()
            .map(|v| v.trim().trim_start_matches(trim).to_lowercase())
            .filter(|v| !v.is_empty())
            .collect();
        if values.is_empty() { Err("a rule has no values".into()) } else { Ok(values) }
    };
    Ok(match rule {
        Rule::Extension { values } => Compiled::Extension(list(values, &['.', '*'])?),
        Rule::Host { values } => Compiled::Host(list(values, &['.', '*'])?),
        Rule::Contains { target, text } if !text.is_empty() => Compiled::Contains(*target, text.to_lowercase()),
        Rule::Contains { .. } => return Err("a rule has no text".into()),
        Rule::Regex { target, pattern } => {
            if pattern.len() > MAX_PATTERN {
                return Err(format!("a regex is longer than {MAX_PATTERN} characters"));
            }
            let regex = regex::RegexBuilder::new(pattern).size_limit(1 << 20).build().map_err(|e| format!("regex: {e}"))?;
            Compiled::Regex(*target, regex)
        }
    })
}

struct Subject<'a> {
    url: &'a str,
    name: &'a str,
    url_lower: String,
    name_lower: String,
    host: Option<String>,
    extension: Option<String>,
}

impl<'a> Subject<'a> {
    fn new(url: &'a str, name: &'a str) -> Self {
        Self {
            url,
            name,
            url_lower: url.to_lowercase(),
            name_lower: name.to_lowercase(),
            host: url::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_lowercase)),
            extension: name.rsplit_once('.').filter(|(stem, _)| !stem.is_empty()).map(|(_, ext)| ext.to_lowercase()),
        }
    }

    fn text(&self, target: Target, lower: bool) -> &str {
        match (target, lower) {
            (Target::Url, false) => self.url,
            (Target::Url, true) => &self.url_lower,
            (Target::Filename, false) => self.name,
            (Target::Filename, true) => &self.name_lower,
        }
    }
}

impl Compiled {
    fn matches(&self, s: &Subject) -> bool {
        match self {
            Compiled::Extension(values) => s.extension.as_ref().is_some_and(|e| values.contains(e)),
            Compiled::Host(values) => s
                .host
                .as_deref()
                .is_some_and(|h| values.iter().any(|v| h == v || h.strip_suffix(v.as_str()).is_some_and(|p| p.ends_with('.')))),
            Compiled::Contains(target, text) => s.text(*target, true).contains(text.as_str()),
            Compiled::Regex(target, regex) => regex.is_match(s.text(*target, false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(id: &str, matching: Matching, rules: Vec<Rule>) -> Category {
        Category { id: id.into(), name: id.into(), matching, rules }
    }

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn rules_tag_tasks() {
        let video = cat("video", Matching::Any, vec![Rule::Extension { values: strings(&[".MP4", "mkv"]) }]);
        let k2s_video = cat(
            "k2s-video",
            Matching::All,
            vec![Rule::Host { values: strings(&["k2s.cc"]) }, Rule::Extension { values: strings(&["mp4"]) }],
        );
        let anime = cat(
            "anime",
            Matching::Any,
            vec![
                Rule::Contains { target: Target::Url, text: "/Anime/".into() },
                Rule::Regex { target: Target::Filename, pattern: r"\[\d{3,4}p\]".into() },
            ],
        );
        let manual = cat("manual", Matching::All, vec![]);
        let c = Classifier::new(&[video, k2s_video, anime, manual]).unwrap();

        assert_eq!(c.tags("https://cdn.example/a.mp4", "a.mp4"), ["video"]);
        assert_eq!(c.tags("https://www.k2s.cc/file/1", "Movie.MP4"), ["video", "k2s-video"], "subdomains, case");
        assert_eq!(c.tags("https://notk2s.cc/file/1", "a.mp4"), ["video"], "not a subdomain");
        assert_eq!(c.tags("https://x/anime/1", "x.zip"), ["anime"]);
        assert_eq!(c.tags("https://x/1", "Show [1080p].zip"), ["anime"]);
        assert!(c.tags("https://x/1", "mp4").is_empty(), "no extension");
        assert_eq!(c.ids().collect::<Vec<_>>(), ["video", "k2s-video", "anime", "manual"]);
    }

    #[test]
    fn mistakes_are_explained() {
        let bad = |rules| Classifier::new(&[cat("x", Matching::Any, rules)]).unwrap_err().to_string();
        assert!(bad(vec![Rule::Regex { target: Target::Url, pattern: "(".into() }]).contains("regex"));
        assert!(bad(vec![Rule::Extension { values: strings(&[" ", "."]) }]).contains("no values"));
        assert!(bad(vec![Rule::Contains { target: Target::Url, text: String::new() }]).contains("no text"));
        let twice = Classifier::new(&[cat("x", Matching::Any, vec![]), cat("x", Matching::Any, vec![])]);
        assert!(twice.unwrap_err().to_string().contains("duplicate"));
    }

    #[test]
    fn wire_shape() {
        let c = cat("v", Matching::All, vec![Rule::Contains { target: Target::Filename, text: "x".into() }]);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::json!({ "id": "v", "name": "v", "match": "all",
                "rules": [{ "kind": "contains", "target": "filename", "text": "x" }] })
        );
    }
}
