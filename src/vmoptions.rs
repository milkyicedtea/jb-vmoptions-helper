use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

pub(crate) fn find_apps() -> HashMap<String, String> {
    let mut apps: HashMap<String, String> = HashMap::new();
    let path_env = std::env::var("PATH").unwrap_or_default();
    let re = Regex::new(r#"^"([^"]+/JetBrains/.+/bin/[^"]+)"\s+"\$@""#).unwrap();

    for dir in path_env.split(':') {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let full_path = entry.path();
            if !full_path.is_file() {
                continue;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = full_path.metadata() {
                    if meta.permissions().mode() & 0o111 == 0 {
                        continue;
                    }
                }
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if apps.contains_key(&name) {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&full_path) {
                for line in content.lines() {
                    if let Some(caps) = re.captures(line.trim()) {
                        apps.insert(name.clone(), caps[1].to_string());
                        break;
                    }
                }
            }
        }
    }
    apps
}

pub(crate) fn resolve_options_path(binary_path: &str) -> Option<PathBuf> {
    let bin_dir = Path::new(binary_path).parent()?;
    let product = Path::new(binary_path).file_name()?.to_string_lossy();

    let candidates = [
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

