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

License: MIT
