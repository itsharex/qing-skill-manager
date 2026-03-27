use std::fs;
use std::path::{Component, Path, PathBuf};

#[cfg(windows)]
fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{}", rest));
    }
    if let Some(rest) = raw.strip_prefix(r"\\?\") {
        return PathBuf::from(rest.to_string());
    }
    path.to_path_buf()
}

#[cfg(not(windows))]
fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let path = strip_windows_verbatim_prefix(path);
    let mut normalized = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(comp),
        }
    }
    normalized
}

/// Windows reserved names that cannot be used as file/directory names
#[cfg(target_os = "windows")]
const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Check if a name is a Windows reserved name
#[cfg(target_os = "windows")]
fn is_windows_reserved_name(name: &str) -> bool {
    let upper = name.to_uppercase();
    // Check exact match
    if WINDOWS_RESERVED_NAMES.contains(&upper.as_str()) {
        return true;
    }
    // Check with extension (e.g., CON.txt, NUL.md)
    if let Some(base) = upper.split('.').next() {
        if WINDOWS_RESERVED_NAMES.contains(&base) {
            return true;
        }
    }
    false
}

pub fn sanitize_dir_name(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '.' {
            out.push('-');
        }
    }

    // Collapse consecutive hyphens into a single hyphen
    let collapsed = collapse_consecutive_hyphens(&out);

    #[cfg(target_os = "windows")]
    let mut result = if collapsed.is_empty() {
        // Generate unique name from non-ASCII input using hash
        if !name.trim().is_empty() {
            format!("skill-{:08x}", hash_name(name))
        } else {
            "skill".to_string()
        }
    } else {
        collapsed.trim_matches('-').to_string()
    };

    #[cfg(not(target_os = "windows"))]
    let result = if collapsed.is_empty() {
        if !name.trim().is_empty() {
            format!("skill-{:08x}", hash_name(name))
        } else {
            "skill".to_string()
        }
    } else {
        collapsed.trim_matches('-').to_string()
    };

    // Windows reserved names check - prefix with underscore to make it safe
    #[cfg(target_os = "windows")]
    if is_windows_reserved_name(&result) {
        result = format!("_{}", result);
    }

    result
}

/// Simple hash for generating unique directory names from non-ASCII input
fn hash_name(name: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for byte in name.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

/// Collapse consecutive hyphens into a single hyphen
fn collapse_consecutive_hyphens(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_hyphen = false;
    for ch in input.chars() {
        if ch == '-' {
            if !prev_was_hyphen {
                result.push(ch);
            }
            prev_was_hyphen = true;
        } else {
            result.push(ch);
            prev_was_hyphen = false;
        }
    }
    result
}

pub fn resolve_canonical(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path)
        .ok()
        .map(|canon| normalize_path(&canon))
}

/// Resolve a path to its canonical form, falling back to normalization if the path doesn't exist.
/// This provides consistent behavior across all path resolution call sites.
pub fn resolve_or_normalize(path: &Path) -> PathBuf {
    resolve_canonical(path).unwrap_or_else(|| normalize_path(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_simple() {
        let path = Path::new("/home/user/docs");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("/home/user/docs"));
    }

    #[test]
    fn test_normalize_path_with_current_dir() {
        let path = Path::new("/home/./user/docs");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("/home/user/docs"));
    }

    #[test]
    fn test_normalize_path_with_parent_dir() {
        let path = Path::new("/home/user/../docs");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("/home/docs"));
    }

    #[test]
    fn test_normalize_path_multiple_parents() {
        let path = Path::new("/home/user/projects/../../docs");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("/home/docs"));
    }

    #[test]
    fn test_sanitize_dir_name_simple() {
        assert_eq!(sanitize_dir_name("My Skill"), "my-skill");
    }

    #[test]
    fn test_sanitize_dir_name_with_special_chars() {
        assert_eq!(sanitize_dir_name("Skill@Home!"), "skillhome");
    }

    #[test]
    fn test_sanitize_dir_name_with_dots() {
        assert_eq!(sanitize_dir_name("skill.v2.test"), "skill-v2-test");
    }

    #[test]
    fn test_sanitize_dir_name_uppercase() {
        assert_eq!(sanitize_dir_name("MY_SKILL"), "my_skill");
    }

    #[test]
    fn test_sanitize_dir_name_empty() {
        assert_eq!(sanitize_dir_name(""), "skill");
    }

    #[test]
    fn test_sanitize_dir_name_only_special_chars() {
        let result = sanitize_dir_name("!!!@@@");
        assert!(result.starts_with("skill-"), "Expected hash-based name for non-empty input, got: {}", result);
    }

    #[test]
    fn test_sanitize_dir_name_trailing_dashes() {
        assert_eq!(sanitize_dir_name("skill-"), "skill");
        assert_eq!(sanitize_dir_name("-skill-"), "skill");
    }

    #[test]
    fn test_sanitize_dir_name_mixed_whitespace() {
        assert_eq!(sanitize_dir_name("my  skill   name"), "my-skill-name");
    }

    #[test]
    fn test_sanitize_dir_name_underscore_preserved() {
        assert_eq!(sanitize_dir_name("my_skill_name"), "my_skill_name");
    }

    #[test]
    fn test_sanitize_dir_name_unicode() {
        let result = sanitize_dir_name("技能测试");
        assert!(result.starts_with("skill-"), "Expected hash-based name, got: {}", result);
        assert_ne!(result, sanitize_dir_name("代码审查"), "Different Unicode names should produce different hashes");
    }

    #[test]
    fn test_sanitize_dir_name_mixed_unicode_and_ascii() {
        let result = sanitize_dir_name("my技能skill");
        assert_eq!(result, "myskill");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_sanitize_dir_name_windows_reserved() {
        assert_eq!(sanitize_dir_name("CON"), "_con");
        assert_eq!(sanitize_dir_name("PRN"), "_prn");
        assert_eq!(sanitize_dir_name("AUX"), "_aux");
        assert_eq!(sanitize_dir_name("NUL"), "_nul");
        assert_eq!(sanitize_dir_name("COM1"), "_com1");
        assert_eq!(sanitize_dir_name("LPT1"), "_lpt1");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_sanitize_dir_name_windows_reserved_with_extension() {
        assert_eq!(sanitize_dir_name("CON.txt"), "_con-txt");
        assert_eq!(sanitize_dir_name("NUL.md"), "_nul-md");
    }
}
