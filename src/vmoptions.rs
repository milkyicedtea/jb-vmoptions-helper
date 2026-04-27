use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

pub(crate) fn find_apps() -> HashMap<String, String> {
    let mut apps: HashMap<String, String> = HashMap::new();
    let path_env = std::env::var("PATH").unwrap_or_default();

    // Shell wrapper regex (Unix: .sh files)
    let re_shell = Regex::new(r#"^"([^"]+/JetBrains/.+/bin/[^"]+)"\s+"\$@""#).unwrap();
    let re_batch = Regex::new(
        r#"(?i)^start\s+""(?:\s+%[a-z0-9_]+%)*\s+(?:"([^"]+\\bin\\[^"]+\.exe)"|([a-z]:\\\S+\\bin\\\S+\.exe))(?:\s|$)"#,
    )
    .unwrap();

    for dir in std::env::split_paths(&path_env) {
        if !dir.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let full_path = entry.path();
            if !full_path.is_file() {
                continue;
            }
            // Cross-platform executable check
            if !is_executable(&full_path) {
                continue;
            }
            let name = full_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
            if apps.contains_key(&name) {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&full_path) {
                let mut found = None;
                for line in content.lines() {
                    // Try Unix shell wrapper first
                    if let Some(caps) = re_shell.captures(line.trim()) {
                        found = Some(caps[1].to_string());
                        break;
                    }
                    if let Some(caps) = re_batch.captures(line.trim()) {
                        let path = caps
                            .get(1)
                            .or_else(|| caps.get(2))
                            .map(|m| m.as_str().to_string());
                        if let Some(path) = path {
                            found = Some(path);
                            break;
                        }
                    }
                }
                if let Some(path) = found {
                    apps.insert(name.clone(), path);
                }
            }
        }
    }
    apps
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = path.metadata() {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    matches!(ext.as_deref(), Some("exe") | Some("bat") | Some("cmd"))
}

pub(crate) fn resolve_options_path(binary_path: &str) -> Option<PathBuf> {
    let binary = Path::new(binary_path);
    let bin_dir = binary.parent()?;
    let binary_name = binary.file_name().and_then(|s| s.to_str())?;
    let product = binary
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(binary_name);
    let product = product.strip_suffix("64").unwrap_or(product);

    let candidates = [
        bin_dir.join(format!("{binary_name}.vmoptions")),
        bin_dir.join(format!("{product}64.exe.vmoptions")),
        bin_dir.join(format!("{product}.exe.vmoptions")),
        bin_dir.join(format!("{product}64.vmoptions")),
        bin_dir.join(format!("{product}.vmoptions")),
    ];

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

pub(crate) fn append_vmoptions(path: &Path, text: &str) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    if content.contains(text) {
        return Ok(false);
    }
    let backup = path.with_extension("vmoptions.bak");
    if !backup.exists() {
        fs::copy(path, &backup)?;
    }
    let new_content = format!("{}\n{}\n", content.trim_end(), text);
    fs::write(path, new_content)?;
    Ok(true)
}

