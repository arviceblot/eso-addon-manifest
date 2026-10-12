# eso_addon_manifest

[![crates.io](https://img.shields.io/crates/v/eso_addon_manifest.svg)](https://crates.io/crates/eso_addon_manifest)
[![codecov](https://codecov.io/gh/arviceblot/eso-addon-manifest/branch/main/graph/badge.svg?token=CEXIFJR0M9)](https://codecov.io/gh/arviceblot/eso-addon-manifest)

A simple ESO AddOn Manifest.txt file parser and validator library

## Installation

Run the following Cargo command in your project directory:

```shell
cargo add eso_addon_manifest
```

## Usage

```rust
use eso_addon_manifest::{AddonManifestParser, AddonManifest};

// let's say you have some addon with the patrial manifest:
// ## Title: AddonName
// ## APIVersion: 101037
// [...]

let parser = AddonManifestParser::default();
let result: AddonManifest = parser.parse("resources/test/AddonName.txt", None).unwrap();
assert_eq!("AddonName".to_string(), result.title);
assert_eq!(101037, result.api_version);
```

## Supported Directives

| Directive | Struct Field | Type |
|---|---|---|
| `Title:` | `title` | `String` |
| `Author:` | `author` | `String` |
| `APIVersion:` | `api_version`, `api_version_2` | `u32`, `Option<u32>` |
| `AddOnVersion:` | `addon_version` | `Option<u32>` |
| `Version:` | `version` | `Option<String>` |
| `DependsOn:` | `depends_on` | `Vec<DependsEntry>` |
| `OptionalDependsOn:` | `optional_depends_on` | `Vec<DependsEntry>` |
| `PCDependsOn:` | `pc_depends_on` | `Vec<DependsEntry>` |
| `ConsoleDependsOn:` | `console_depends_on` | `Vec<DependsEntry>` |
| `Description:` | `description` | `Option<String>` |
| `IsLibrary:` | `is_library` | `Option<bool>` |
| `SavedVariables:` | `saved_variables` | `Vec<String>` |
| `DisableSavedVariablesAutoSaving:` | `disable_saved_variables_auto_saving` | `Option<bool>` |

Unrecognized directives produce a warning rather than an error, so non-standard directives like `Credits:` or `Contributors:` won't cause parsing to fail.

License: MIT
