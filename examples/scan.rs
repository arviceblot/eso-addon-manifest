use std::path::PathBuf;

use eso_addon_manifest::AddonManifestParser;

fn find_manifests(dir: &str, depth: u32, max_depth: u32, results: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "txt" || ext == "addon" {
                        results.push(path);
                    }
                }
            } else if path.is_dir() {
                find_manifests(&path.to_string_lossy(), depth + 1, max_depth, results);
            }
        }
    }
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());

    let parser = AddonManifestParser::default();
    let mut files = Vec::new();
    find_manifests(&dir, 1, 2, &mut files);
    files.sort();

    let mut error_count = 0usize;
    let mut warn_count = 0usize;
    let mut ok_count = 0usize;
    let mut total_warnings = 0usize;

    for file in &files {
        let path_str = file.to_string_lossy();
        let rel = file.strip_prefix(&dir).unwrap_or(file).to_string_lossy();
        match parser.parse(&path_str, Some(true)) {
            Ok(manifest) => {
                let e = manifest.errors.len();
                let w = manifest.warnings.len();
                total_warnings += w;
                if e > 0 {
                    error_count += 1;
                    println!("FAIL: {}", rel);
                    for err in &manifest.errors {
                        println!("  ERROR: {err}");
                    }
                    if w > 0 {
                        for warn in &manifest.warnings {
                            println!("  WARN:  {warn}");
                        }
                    }
                } else if w > 0 {
                    warn_count += 1;
                    println!("WARN: {} ({} warning(s))", rel, w);
                    for warn in &manifest.warnings {
                        println!("  WARN:  {warn}");
                    }
                } else {
                    ok_count += 1;
                    println!(
                        "OK:   {} — \"{}\" (API {})",
                        rel, manifest.title, manifest.api_version
                    );
                }
            }
            Err(e) => {
                error_count += 1;
                println!("ERR:  {} — {}", rel, e);
            }
        }
    }

    println!("\n--- Summary ---");
    println!("OK:     {}", ok_count);
    println!(
        "Warn:   {} (total warnings: {})",
        warn_count, total_warnings
    );
    println!("Errors: {}", error_count);
}
