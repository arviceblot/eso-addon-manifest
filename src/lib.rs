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
//! let result: AddonManifest = parser.parse("resources/test/AddonName.txt", None).unwrap();
//! assert_eq!("AddonName".to_string(), result.title);
//! assert_eq!(101037, result.api_version);
//! ```
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links
)]
mod error;

use std::{fs::File, io::Read};

use error::{ManifestError, Result};
use regex::Regex;

static RE_DIRECTIVE: &str = r#"^## (?P<directive>.*): (?P<value>.*)"#;
static RE_DEPENDS: &str = r#"^(?P<name>.+?)(([<=>]+)(?P<version>.*))?$"#;
static RE_UNPRETTY: &str = r#"\|c\S{6}(?<text>[^\|]*)\|r"#;

enum LineType {
    Directive,
    Comment,
    Blank,
    Data,
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
    /// PC-only dependencies; only effective in .addon manifests. Same format as DependsOn.
    pub pc_depends_on: Vec<DependsEntry>,
    /// Console-only dependencies; only effective in .addon manifests. Same format as DependsOn.
    pub console_depends_on: Vec<DependsEntry>,
    /// Space-separated list of names for data objects persisted to disk between UI loads
    pub saved_variables: Vec<String>,
    /// A short description of the add-on for human display (e.g. "A helpful addon for managing inventory")
    pub description: Option<String>,
    /// Set to true to disable the game's automatic saving of this addon's saved variables
    pub disable_saved_variables_auto_saving: Option<bool>,
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
            && self.pc_depends_on == other.pc_depends_on
            && self.console_depends_on == other.console_depends_on
            && self.saved_variables == other.saved_variables
            && self.description == other.description
            && self.disable_saved_variables_auto_saving == other.disable_saved_variables_auto_saving
            && self.is_library == other.is_library
    }
}

/// Parser helper struct
#[derive(Debug)]
pub struct AddonManifestParser {
    re_directive: Regex,
    re_depend: Regex,
    re_unpretty: Regex,
}
impl Default for AddonManifestParser {
    fn default() -> Self {
        Self {
            re_directive: Regex::new(RE_DIRECTIVE).unwrap(),
            re_depend: Regex::new(RE_DEPENDS).unwrap(),
            re_unpretty: Regex::new(RE_UNPRETTY).unwrap(),
        }
    }
}

impl AddonManifestParser {
    /// Parse a given file path into an AddonManifest result
    pub fn parse(&self, path: &str, full_validate: Option<bool>) -> Result<AddonManifest> {
        let full_validate = full_validate.unwrap_or_default();
        let mut file = File::open(path).map_err(ManifestError::FileIOError)?;

        let mut result = AddonManifest {
            title: "".to_string(),
            author: "".to_string(),
            api_version: 0,
            ..Default::default()
        };

        let mut buf = vec![];
        let _ = file.read_to_end(&mut buf);
        let contents = String::from_utf8_lossy(&buf);
        for line in contents.lines() {
            match self.parse_line(line, &mut result, full_validate) {
                Ok(_) => {}
                Err(error) => {
                    result.errors.push(error);
                }
            }
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
        Ok(result)
    }

    fn parse_line(
        &self,
        line: &str,
        result: &mut AddonManifest,
        full_validate: bool,
    ) -> Result<()> {
        // determine line type
        let line_type = LineType::from_line(line);
        match line_type {
            LineType::Directive => {
                let dir_captures = self.re_directive.captures(line);
                match dir_captures {
                    Some(captures) => {
                        if full_validate && line.len() > 301 {
                            result.errors.push(ManifestError::LineLength(line.len()));
                        }

                        // validate captures
                        let directive_cap = captures.name("directive");
                        if directive_cap.is_none() {
                            return Err(ManifestError::InvalidDirective(line.to_string()));
                        }
                        let directive = directive_cap.unwrap().as_str();
                        let value_cap = captures.name("value");
                        if value_cap.is_none() {
                            return Err(ManifestError::InvalidDirective(line.to_string()));
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
                                result.title =
                                    self.re_unpretty.replace_all(value, "$text").to_string();
                            }
                            "Author" => {
                                result.author = value.to_string();
                            }
                            "APIVersion" => {
                                if value.contains(' ') {
                                    // we have to supported version
                                    let values: Vec<&str> = value.split(' ').collect();
                                    result.api_version =
                                        values[0].parse().map_err(ManifestError::ParseIntError)?;
                                    result.api_version_2 = Some(
                                        values[1].parse().map_err(ManifestError::ParseIntError)?,
                                    );
                                } else {
                                    result.api_version =
                                        value.parse().map_err(ManifestError::ParseIntError)?;
                                }
                            }
                            "AddOnVersion" => {
                                result.addon_version =
                                    Some(value.parse().map_err(ManifestError::ParseIntError)?);
                            }
                            "Version" => {
                                result.version = Some(value.to_string());
                            }
                            "DependsOn" => {
                                let depends = self.parse_depends(value)?;
                                result.depends_on.extend(depends);
                            }
                            "OptionalDependsOn" => {
                                let depends = self.parse_depends(value)?;
                                result.optional_depends_on.extend(depends);
                            }
                            "PCDependsOn" => {
                                let depends = self.parse_depends(value)?;
                                result.pc_depends_on.extend(depends);
                            }
                            "ConsoleDependsOn" => {
                                let depends = self.parse_depends(value)?;
                                result.console_depends_on.extend(depends);
                            }
                            "SavedVariables" => {
                                let vars: Vec<String> =
                                    value.split_whitespace().map(String::from).collect();
                                result.saved_variables.extend(vars);
                            }
                            "Description" => {
                                result.description = Some(value.to_string());
                            }
                            "DisableSavedVariablesAutoSaving" => {
                                result.disable_saved_variables_auto_saving = Some(value == "1");
                            }
                            "IsLibrary" => {
                                result.is_library =
                                    Some(value.parse().map_err(ManifestError::ParseBoolError)?);
                            }
                            _ => {
                                // unmatched directives are not necessarily an error, see: Credits, Contributors, etc.
                                result
                                    .warnings
                                    .push(ManifestError::UnmappedDirective(directive.to_string()));
                            }
                        }
                    }
                    None => {
                        // matches directive line type but does not match regex
                        result
                            .errors
                            .push(ManifestError::InvalidDirective(line.to_string()))
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
        };
        Ok(())
    }

    fn parse_depends(&self, line: &str) -> Result<Vec<DependsEntry>> {
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
                    depends_entry.version = Some(
                        version
                            .as_str()
                            .parse()
                            .map_err(ManifestError::ParseIntError)?,
                    );
                }

                if !depends_entry.title.is_empty() {
                    result.push(depends_entry);
                }
            } else {
                // invalid depends entry
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf, vec};

    use crate::{AddonManifest, AddonManifestParser, DependsEntry};

    macro_rules! parse_depends_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let parser = AddonManifestParser::default();
                    let result = parser.parse_depends(input).unwrap();
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
                        parser.parse_line(line, &mut result, false).unwrap();
                    }
                    assert_eq!(expected, result);
                }
            )*
        }
    }

    macro_rules! parse_lines_tests_validation {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let parser = AddonManifestParser::default();
                    let mut result = AddonManifest::default();
                    for line in input.iter() {
                        parser.parse_line(line, &mut result, true).unwrap();
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
        test_parse_line_fomatted_title: (
            vec!["## Title: |cEFEBBELibLibrary|r"],
            AddonManifest {
                title: "LibLibrary".to_string(),
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
        test_parse_line_single_api_version: (
            vec!["## APIVersion: 101000"],
            AddonManifest {
                api_version: 101000,
                ..Default::default()
            }
        ),
        test_parse_other_lines: (
            vec![
                "## OptionalDependsOn: TheLib",
                "## IsLibrary: true",
                "## Credits: look mom, I finally did it"
            ],
            AddonManifest {
                optional_depends_on: vec![DependsEntry {
                    title: "TheLib".to_string(),
                    version: None
                }],
                is_library: Some(true),
                ..Default::default()
            }
        ),
        test_parse_line_description: (
            vec!["## Description: A helpful addon for managing inventory"],
            AddonManifest {
                description: Some("A helpful addon for managing inventory".to_string()),
                ..Default::default()
            }
        ),
        test_parse_line_description_empty: (
            vec!["## Description: "],
            AddonManifest {
                description: Some("".to_string()),
                ..Default::default()
            }
        ),
        test_parse_line_pc_depends_on: (
            vec!["## PCDependsOn: LibAddonMenu-2.0>=40"],
            AddonManifest {
                pc_depends_on: vec![DependsEntry {
                    title: "LibAddonMenu-2.0".to_string(),
                    version: Some(40),
                }],
                ..Default::default()
            }
        ),
        test_parse_line_console_depends_on: (
            vec!["## ConsoleDependsOn: LibHarvensAddonSettings>=20004"],
            AddonManifest {
                console_depends_on: vec![DependsEntry {
                    title: "LibHarvensAddonSettings".to_string(),
                    version: Some(20004),
                }],
                ..Default::default()
            }
        ),
        test_parse_line_saved_variables: (
            vec!["## SavedVariables: MyAddonSavedVars MyAddonSettings"],
            AddonManifest {
                saved_variables: vec![
                    "MyAddonSavedVars".to_string(),
                    "MyAddonSettings".to_string(),
                ],
                ..Default::default()
            }
        ),
        test_parse_line_disable_saved_variables_auto_saving: (
            vec!["## DisableSavedVariablesAutoSaving: 1"],
            AddonManifest {
                disable_saved_variables_auto_saving: Some(true),
                ..Default::default()
            }
        ),
        test_parse_line_disable_saved_variables_auto_saving_disabled: (
            vec!["## DisableSavedVariablesAutoSaving: 0"],
            AddonManifest {
                disable_saved_variables_auto_saving: Some(false),
                ..Default::default()
            }
        ),
        test_parse_lines: (
            vec![
                "## Title: LibLibrary",
                "## Description: A library for addon development",
                "## DependsOn: CustomLib>=4 OtherLib",
                "## Version: 1.20",
                "## AddOnVersion: 27",
                "## APIVersion: 101000 101001",
                "## Author: TheAuthor",
                "",
                "; a comment",
                "#another type of comment",
                "some_script.lua",
            ],
            AddonManifest {
                title: "LibLibrary".to_string(),
                description: Some("A library for addon development".to_string()),
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

    parse_lines_tests_validation! {
        test_title_length: (
            vec!["## Title: This is a really long title, wow look how long it is. So great with many words. Who could forget such a beautiful title. It's like it never ends becuase it does not want to. It will consume the world."],
            AddonManifest {
                title: "This is a really long title, wow look how long it is. So great with many words. Who could forget such a beautiful title. It's like it never ends becuase it does not want to. It will consume the world.".to_string(),
                ..Default::default()
            }
        ),
        test_unmapped_directive_stores_name: (
            vec!["## Credits: SomeAuthor"],
            AddonManifest {
                ..Default::default()
            }
        ),
    }

    #[test]
    fn test_parse_file() {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let file_path: PathBuf = [dir.as_str(), "resources", "test", "AddonName.txt"]
            .iter()
            .collect();
        let parser = AddonManifestParser::default();
        let manifest = parser
            .parse(file_path.to_str().unwrap(), Some(true))
            .unwrap();
        assert_eq!(manifest.errors.len(), 1);
    }

    #[test]
    fn test_unmapped_directive_warning_stores_name() {
        let parser = AddonManifestParser::default();
        let mut result = AddonManifest::default();
        parser
            .parse_line("## Credits: SomeAuthor", &mut result, true)
            .unwrap();
        assert_eq!(result.warnings.len(), 1);
        // The warning should contain the directive NAME, not the value.
        assert_eq!(
            result.warnings[0].to_string(),
            "unmapped directive: Credits"
        );
    }

    #[test]
    fn test_parse_file_with_description() {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let file_path: PathBuf = [
            dir.as_str(),
            "resources",
            "test",
            "AddonWithDescription.txt",
        ]
        .iter()
        .collect();
        let parser = AddonManifestParser::default();
        let manifest = parser
            .parse(file_path.to_str().unwrap(), Some(true))
            .unwrap();
        assert_eq!(manifest.title, "AddonWithDescription");
        assert_eq!(manifest.api_version, 101037);
        assert_eq!(manifest.author, "arviceblot");
        assert_eq!(
            manifest.description,
            Some("An addon that tests the description field parsing".to_string())
        );
        // No errors expected since all required fields are present and valid
        assert!(manifest.errors.is_empty());
    }

    #[test]
    fn test_parse_file_full_features() {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let file_path: PathBuf = [dir.as_str(), "resources", "test", "FullFeatureAddon.txt"]
            .iter()
            .collect();
        let parser = AddonManifestParser::default();
        let manifest = parser
            .parse(file_path.to_str().unwrap(), Some(true))
            .unwrap();

        // Basic fields
        assert_eq!(manifest.title, "FullFeatureAddon");
        assert_eq!(manifest.api_version, 101048);
        assert_eq!(manifest.addon_version, Some(5));
        assert_eq!(manifest.author, "TestAuthor");
        assert_eq!(manifest.version, Some("1.0.0".to_string()));
        assert_eq!(manifest.is_library, Some(false));
        assert_eq!(
            manifest.description,
            Some("An addon that exercises all supported manifest directives".to_string())
        );

        // DisableSavedVariablesAutoSaving
        assert_eq!(manifest.disable_saved_variables_auto_saving, Some(true));

        // DependsOn
        assert_eq!(manifest.depends_on.len(), 1);
        assert_eq!(manifest.depends_on[0].title, "LibAddonMenu-2.0");
        assert_eq!(manifest.depends_on[0].version, Some(41));

        // OptionalDependsOn
        assert_eq!(manifest.optional_depends_on.len(), 1);
        assert_eq!(manifest.optional_depends_on[0].title, "Skyshards");

        // PCDependsOn
        assert_eq!(manifest.pc_depends_on.len(), 1);
        assert_eq!(manifest.pc_depends_on[0].title, "LibCustomPC");
        assert_eq!(manifest.pc_depends_on[0].version, Some(1));

        // ConsoleDependsOn
        assert_eq!(manifest.console_depends_on.len(), 1);
        assert_eq!(manifest.console_depends_on[0].title, "LibCustomConsole");
        assert_eq!(manifest.console_depends_on[0].version, Some(2));

        // SavedVariables
        assert_eq!(manifest.saved_variables.len(), 2);
        assert_eq!(manifest.saved_variables[0], "FullFeatureAddonSavedVars");
        assert_eq!(manifest.saved_variables[1], "FullFeatureAddonSettings");

        // No errors expected
        assert!(manifest.errors.is_empty());
    }
}
