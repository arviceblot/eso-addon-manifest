//! A simple ESO AddOn Manifest.txt file parser and validator
//!
//! # Usage
//! ```rust
//! use eso_addon_manifest::{AddonManifestParser, AddonManifest};
//!
//! // let's say you have some addon with the patrial manifest:
//! // ## Title: AddonName
//! // ## APIVersion: 101037
//! // [...]
//!
//! let parser = AddonManifestParser::default();
//! let result: AddonManifest = parser.parse("some/file/path/AddonName.txt".to_string());
//! assert_eq!("AddonName".to_string(), result.title);
//! assert_eq!(101037, result.addon_version);
//! ```
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    broken_intra_doc_links
)]
mod error;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use error::{ManifestError, Result};
use regex::Regex;

static RE_DIRECTIVE: &str = r#"^## (?P<directive>.*): (?P<value>.*)"#;
static RE_DEPENDS: &str = r#"^(?P<name>.+?)(([<=>]+)(?P<version>.*))?$"#;

enum LineType {
    Directive,
    Comment,
    Blank,
    Data,
    Unknown,
}
impl LineType {
    pub fn from_line(line: &str) -> Self {
        if line.starts_with("## ") {
            Self::Directive
        } else if line.starts_with('#') || line.starts_with(';') {
            Self::Comment
        } else if line.trim().is_empty() {
            Self::Blank
        } else {
            Self::Data
        }
    }
}

/// AddOn Depenency data
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DependsEntry {
    /// Dependent addon title
    pub title: String,
    /// Optional dependent addon version
    pub version: Option<u32>,
}

/// Manifest file data store
///
/// Validation data provided by: [ESOUI Wiki](https://wiki.esoui.com/Addon_manifest_(.txt)_format)
#[derive(Debug, Default)]
pub struct AddonManifest {
    /// Addon title, a character string for human display (e.g. SkyShards)
    pub title: String,
    /// The author name String of the addon
    pub author: String,
    /// A six digit value, the same value as the ESO API for each release (e.g. 100026). In the same line there can be up to 2 APIVersions after another, separated by a space, to support both of them (e.g. 100026 100027)
    pub api_version: u32,
    /// Optional second supported APIVersion
    pub api_version_2: Option<u32>,
    /// A positive integer value (e.g. 1,2,3,etc. but not 3.4 nor -5 nor r5)
    pub addon_version: Option<u32>,
    /// A version identifier for ESOUI and/or Minion (e.g. 2.0.2) to separate add-on releases and/or updates
    pub version: Option<String>,
    /// A space separated name list of add-ons/libraries that your add-on needs to run correctly (e.g. LibAddonMenu-2.0 LibDialog). If any addon/library in this line is missing your adon won't load!
    pub depends_on: Vec<DependsEntry>,
    /// A name list similar to DependsOn: but the add-ons in this list will not prevent your add-on from running. Use this to assure other addons listed here are loaded before your addon (e.g. AddonName1 AddonName2).
    pub optional_depends_on: Vec<DependsEntry>,
    /// false or not present : if this add-on is not a library or support add-on; true : if this add-on is a library or support add-on
    pub is_library: Option<bool>,
    /// Vec of errors produced during import or full validation
    pub errors: Vec<ManifestError>,
    /// Vec of warnings (stored as error type) during import or full validation
    pub warnings: Vec<ManifestError>,
}
impl PartialEq for AddonManifest {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.author == other.author
            && self.api_version == other.api_version
            && self.api_version_2 == other.api_version_2
            && self.addon_version == other.addon_version
            && self.version == other.version
            && self.depends_on == other.depends_on
            && self.optional_depends_on == other.optional_depends_on
            && self.is_library == other.is_library
    }
}

/// Parser helper struct
#[derive(Debug)]
pub struct AddonManifestParser {
    re_directive: Regex,
    re_depend: Regex,
}
impl Default for AddonManifestParser {
    fn default() -> Self {
        Self {
            re_directive: Regex::new(RE_DIRECTIVE).unwrap(),
            re_depend: Regex::new(RE_DEPENDS).unwrap(),
        }
    }
}

impl AddonManifestParser {
    /// Parse a given file path into an AddonManifest result
    pub fn parse(&mut self, path: String, full_validate: Option<bool>) -> Result<AddonManifest> {
        let full_validate = full_validate.unwrap_or_default();
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        let mut result = AddonManifest {
            title: "".to_string(),
            author: "".to_string(),
            api_version: 0,
            ..Default::default()
        };

        for line in reader.lines() {
            let line = line.map_err(ManifestError::ReadLineError).unwrap();
            self.parse_line(line, &mut result, full_validate);
        }

        if full_validate {
            if result.title.trim().is_empty() {
                result
                    .errors
                    .push(ManifestError::MissingDirective("Title".to_string()));
            }
            if result.author.trim().is_empty() {
                result
                    .errors
                    .push(ManifestError::MissingDirective("Author".to_string()));
            }
            if result.api_version < 100003 {
                result
                    .errors
                    .push(ManifestError::ApiMinimumVersion(result.api_version))
            }
        }
        if !result.errors.is_empty() {
            // return Err(result);
        }
        Ok(result)
    }

    fn parse_line(&self, line: String, result: &mut AddonManifest, full_validate: bool) {
        // determine line type
        let line_type = LineType::from_line(&line);
        match line_type {
            LineType::Directive => {
                let dir_captures = self.re_directive.captures(&line);
                match dir_captures {
                    Some(captures) => {
                        if full_validate && line.len() > 301 {
                            result.errors.push(ManifestError::LineLength(line.len()));
                        }

                        // validate captures
                        let directive_cap = captures.name("directive");
                        if directive_cap.is_none() {
                            result.errors.push(ManifestError::InvalidDirective(line));
                            return;
                        }
                        let directive = directive_cap.unwrap().as_str();
                        let value_cap = captures.name("value");
                        if value_cap.is_none() {
                            result.errors.push(ManifestError::InvalidDirective(line));
                            return;
                        }
                        let value = value_cap.unwrap().as_str();

                        // check directive type
                        match directive {
                            "Title" => {
                                if full_validate {
                                    let char_len = value.chars().count();
                                    if char_len > 64 {
                                        result.errors.push(ManifestError::TitleLength(char_len));
                                    }
                                }
                                result.title = value.to_string();
                            }
                            "Author" => {
                                result.author = value.to_string();
                            }
                            "APIVersion" => {
                                if value.contains(' ') {
                                    // we have to suppported version
                                    let values: Vec<u32> =
                                        value.split(' ').map(|x| x.parse().unwrap()).collect();
                                    result.api_version = values[0];
                                    result.api_version_2 = Some(values[1]);
                                } else {
                                    result.api_version = value.parse().unwrap();
                                }
                            }
                            "AddOnVersion" => {
                                result.addon_version = Some(value.parse().unwrap());
                            }
                            "Version" => {
                                result.version = Some(value.to_string());
                            }
                            "DependsOn" => {
                                let depends = self.parse_depends(value);
                                result.depends_on.extend(depends);
                            }
                            "OptionalDependsOn" => {
                                let depends = self.parse_depends(value);
                                result.optional_depends_on.extend(depends);
                            }
                            "IsLibrary" => {
                                result.is_library = Some(value.parse().unwrap());
                            }
                            _ => {
                                // unmatched directives are not necessarily an error, see: Credits, Contributors, etc.
                                result
                                    .warnings
                                    .push(ManifestError::UnmappedDirective(value.to_string()));
                            }
                        }
                    }
                    None => {
                        // matches directive line type but does not match regex
                        result.errors.push(ManifestError::InvalidDirective(line))
                    }
                }
            }
            LineType::Comment => {
                if full_validate {
                    let char_len = line.chars().count();
                    if char_len > 1024 {
                        result.errors.push(ManifestError::CommentLength(char_len));
                    }
                }
            }
            LineType::Blank => {}
            LineType::Data => {}
            LineType::Unknown => {}
        }
    }

    fn parse_depends(&self, line: &str) -> Vec<DependsEntry> {
        let mut result = vec![];
        let values: Vec<&str> = line.split(' ').collect();
        for val in values.iter() {
            if let Some(captures) = self.re_depend.captures(val) {
                let mut depends_entry = DependsEntry::default();
                // check name and maybe version
                if let Some(name) = captures.name("name") {
                    depends_entry.title = name.as_str().to_string();
                } else {
                    // handle bad entry error
                }
                if let Some(version) = captures.name("version") {
                    depends_entry.version = Some(version.as_str().parse().unwrap());
                }

                if !depends_entry.title.is_empty() {
                    result.push(depends_entry);
                }
            } else {
                // invalid depends entry
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{AddonManifest, AddonManifestParser, DependsEntry};

    macro_rules! parse_depends_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let parser = AddonManifestParser::default();
                    let result = parser.parse_depends(input);
                    assert_eq!(expected, result);
                }
            )*
        }
    }

    macro_rules! parse_lines_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let parser = AddonManifestParser::default();
                    let mut result = AddonManifest::default();
                    for line in input.iter() {
                        parser.parse_line(line.to_string(), &mut result, false);
                    }
                    assert_eq!(expected, result);
                }
            )*
        }
    }

    #[test]
    fn test_default() {
        AddonManifestParser::default();
    }

    #[test]
    fn test_depends_ne_version_none() {
        assert_ne!(
            DependsEntry {
                title: "".to_string(),
                version: Some(1)
            },
            DependsEntry {
                title: "".to_string(),
                version: None
            }
        );
    }

    parse_depends_tests! {
        test_parse_depend_single: (
            "LibLibrary",
            vec![
                DependsEntry {
                    title: "LibLibrary".to_string(),
                    version: None,
            }]),
        test_parse_depend_multiple: (
            "LibLibrary LibOther",
            vec![
                DependsEntry {
                    title: "LibLibrary".to_string(),
                    version: None,
                },
                DependsEntry {
                    title: "LibOther".to_string(),
                    version: None,
                },
            ]),
        test_parse_depend_version: (
            "LibLibrary>=20",
            vec![
                DependsEntry {
                    title: "LibLibrary".to_string(),
                    version: Some(20),
            }]),
        test_parse_depend_version_multiple: (
            "LibLibrary>=10 CustomAddon LibOther<=5",
            vec![
                DependsEntry {
                    title: "LibLibrary".to_string(),
                    version: Some(10),
                },
                DependsEntry {
                    title: "CustomAddon".to_string(),
                    version: None,
                },
                DependsEntry {
                    title: "LibOther".to_string(),
                    version: Some(5),
                },
            ]),
    }

    parse_lines_tests! {
        test_parse_line_directive: (
            vec!["## Title: SomeTitle"],
            AddonManifest {
                title: "SomeTitle".to_string(),
                ..Default::default()
            }
        ),
        test_parse_line_depends: (
            vec!["## DependsOn: LibLibrary"],
            AddonManifest {
                depends_on: vec![DependsEntry {
                    title: "LibLibrary".to_string(),
                    version: None,
                }],
                ..Default::default()
            }
        ),
        test_parse_lines: (
            vec![
                "## Title: LibLibrary",
                "## DependsOn: CustomLib>=4 OtherLib",
                "## Version: 1.20",
                "## AddOnVersion: 27",
                "## APIVersion: 101000 101001",
                "## Author: TheAuthor",
            ],
            AddonManifest {
                title: "LibLibrary".to_string(),
                depends_on: vec![
                    DependsEntry {
                        title: "CustomLib".to_string(),
                        version: Some(4)
                    },
                    DependsEntry {
                        title: "OtherLib".to_string(),
                        version: None
                    }
                ],
                version: Some("1.20".to_string()),
                addon_version: Some(27),
                api_version: 101000,
                api_version_2: Some(101001),
                author: "TheAuthor".to_string(),
                ..Default::default()
            }
        ),
    }
}
