# eso_addon_manifest

A simple ESO AddOn Manifest.txt file parser and validator

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
