use crate::output::estimate_tokens;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefFile {
    pub rel_path: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub lines: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptFile {
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFile {
    pub rel_path: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFileEntry {
    pub rel_path: String,
    pub category: String,
    pub bytes: u64,
    pub lines: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStats {
    pub total_files: usize,
    pub total_bytes: u64,
    pub total_tokens: usize,
    pub skill_md_lines: usize,
    pub skill_md_bytes: u64,
    pub skill_md_tokens: usize,
    pub ref_count: usize,
    pub ref_bytes: u64,
    pub ref_tokens: usize,
    pub script_count: usize,
    pub asset_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub dir_path: PathBuf,
    pub skill_md_path: Option<PathBuf>,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub allowed_tools: Vec<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub frontmatter_raw: Option<String>,
    pub body: String,
}

impl Skill {
    pub fn from_dir(dir: &Path) -> Option<Self> {
        if !dir.is_dir() {
            return None;
        }

        let folder_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Standard SKILL.md or skill.md
        let candidates = [dir.join("SKILL.md"), dir.join("skill.md")];
        let skill_md = candidates.into_iter().find(|p| p.is_file());

        let (name, description, license, compatibility, allowed_tools, dependencies, metadata, fm_raw, body) =
            if let Some(ref path) = skill_md {
                let content = fs::read_to_string(path).ok().unwrap_or_default();
                let (fm, b) = parse_frontmatter(&content);
                let (n, desc, lic, comp, tools, deps, meta) = parse_yaml_metadata(&fm, &folder_name);
                (n, desc, lic, comp, tools, deps, meta, Some(fm), b)
            } else {
                (folder_name, String::new(), None, None, Vec::new(), Vec::new(), HashMap::new(), None, String::new())
            };

        Some(Self {
            name,
            dir_path: dir.to_path_buf(),
            skill_md_path: skill_md,
            description,
            license,
            compatibility,
            allowed_tools,
            dependencies,
            metadata,
            frontmatter_raw: fm_raw,
            body,
        })
    }

    pub fn references_dir(&self) -> PathBuf {
        self.dir_path.join("references")
    }

    pub fn scripts_dir(&self) -> PathBuf {
        self.dir_path.join("scripts")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.dir_path.join("assets")
    }

    pub fn list_references(&self) -> Vec<RefFile> {
        let mut results = Vec::new();
        let ref_dir = self.references_dir();
        if !ref_dir.is_dir() {
            return results;
        }

        for entry in WalkDir::new(&ref_dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path();
                if is_ignored_path(p) {
                    continue;
                }
                let rel = match p.strip_prefix(&ref_dir) {
                    Ok(r) => r.to_string_lossy().to_string(),
                    Err(_) => p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                };
                let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let (lines, tokens) = if let Ok(s) = fs::read_to_string(p) {
                    (s.lines().count(), estimate_tokens(&s))
                } else {
                    (0, (bytes / 4) as usize)
                };

                results.push(RefFile {
                    rel_path: rel,
                    path: p.to_path_buf(),
                    bytes,
                    lines,
                    tokens,
                });
            }
        }
        results.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        results
    }

    pub fn list_scripts(&self) -> Vec<ScriptFile> {
        let mut results = Vec::new();
        let sdir = self.scripts_dir();
        if !sdir.is_dir() {
            return results;
        }

        for entry in WalkDir::new(&sdir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path();
                if is_ignored_path(p) {
                    continue;
                }
                let name = match p.strip_prefix(&sdir) {
                    Ok(r) => r.to_string_lossy().to_string(),
                    Err(_) => p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                };
                let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                results.push(ScriptFile {
                    name,
                    path: p.to_path_buf(),
                    bytes,
                });
            }
        }
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    pub fn list_assets(&self) -> Vec<AssetFile> {
        let mut results = Vec::new();
        let adir = self.assets_dir();
        if !adir.is_dir() {
            return results;
        }

        for entry in WalkDir::new(&adir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path();
                if is_ignored_path(p) {
                    continue;
                }
                let rel = match p.strip_prefix(&adir) {
                    Ok(r) => r.to_string_lossy().to_string(),
                    Err(_) => p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                };
                let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                results.push(AssetFile {
                    rel_path: rel,
                    path: p.to_path_buf(),
                    bytes,
                });
            }
        }
        results.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        results
    }

    pub fn list_all_files(&self) -> Vec<SkillFileEntry> {
        let mut entries = Vec::new();
        for entry in WalkDir::new(&self.dir_path).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path();
                if is_ignored_path(p) {
                    continue;
                }
                let rel = match p.strip_prefix(&self.dir_path) {
                    Ok(r) => r.to_string_lossy().to_string(),
                    Err(_) => p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                };
                let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let category = if rel.eq_ignore_ascii_case("skill.md") {
                    "instruction".to_string()
                } else if rel.starts_prefix("references/") {
                    "reference".to_string()
                } else if rel.starts_prefix("scripts/") {
                    "script".to_string()
                } else if rel.starts_prefix("assets/") {
                    "asset".to_string()
                } else if rel.starts_prefix("examples/") {
                    "example".to_string()
                } else {
                    "other".to_string()
                };

                let (lines, tokens) = if let Ok(s) = fs::read_to_string(p) {
                    (s.lines().count(), estimate_tokens(&s))
                } else {
                    (0, (bytes / 4) as usize)
                };

                entries.push(SkillFileEntry {
                    rel_path: rel,
                    category,
                    bytes,
                    lines,
                    tokens,
                });
            }
        }
        entries.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        entries
    }

    pub fn stats(&self) -> SkillStats {
        let refs = self.list_references();
        let scripts = self.list_scripts();
        let assets = self.list_assets();

        let ref_count = refs.len();
        let ref_bytes: u64 = refs.iter().map(|r| r.bytes).sum();
        let ref_tokens: usize = refs.iter().map(|r| r.tokens).sum();

        let (md_lines, md_bytes, md_tokens) = if let Some(ref p) = self.skill_md_path {
            if let Ok(meta) = fs::metadata(p) {
                let b = fs::read_to_string(p).unwrap_or_default();
                (b.lines().count(), meta.len(), estimate_tokens(&b))
            } else {
                (0, 0, 0)
            }
        } else {
            (0, 0, 0)
        };

        let all_files = self.list_all_files();
        let total_files = all_files.len();
        let total_bytes: u64 = all_files.iter().map(|f| f.bytes).sum();
        let total_tokens: usize = all_files.iter().map(|f| f.tokens).sum();

        SkillStats {
            total_files,
            total_bytes,
            total_tokens,
            skill_md_lines: md_lines,
            skill_md_bytes: md_bytes,
            skill_md_tokens: md_tokens,
            ref_count,
            ref_bytes,
            ref_tokens,
            script_count: scripts.len(),
            asset_count: assets.len(),
        }
    }
}

fn is_ignored_path(p: &Path) -> bool {
    let s = p.to_string_lossy();
    if s.contains("/__pycache__")
        || s.contains("/.git/")
        || s.contains("/.cache/")
        || s.contains("/node_modules/")
    {
        return true;
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if ext == "pyc" || ext == "pyo" {
            return true;
        }
    }
    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
        if name == ".DS_Store" {
            return true;
        }
    }
    false
}

trait StartsPrefix {
    fn starts_prefix(&self, prefix: &str) -> bool;
}

impl StartsPrefix for str {
    fn starts_prefix(&self, prefix: &str) -> bool {
        self.starts_with(prefix)
    }
}

fn parse_frontmatter(content: &str) -> (String, String) {
    let trimmed = content.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end_idx) = rest.find("\n---") {
            let fm = &rest[..end_idx];
            let after = &rest[end_idx + 4..];
            let body = after.trim_start_matches('\r').trim_start_matches('\n');
            return (fm.trim().to_string(), body.to_string());
        }
    }
    (String::new(), content.to_string())
}


// Lightweight manual frontmatter parser to avoid heavy yaml dependency
#[allow(clippy::type_complexity)]
fn parse_yaml_metadata(
    fm: &str,
    fallback_name: &str,
) -> (
    String,
    String,
    Option<String>,
    Option<String>,
    Vec<String>,
    Vec<String>,
    HashMap<String, String>,
) {
    let mut name = fallback_name.to_string();
    let mut desc = String::new();
    let mut license = None;
    let mut comp = None;
    let mut tools = Vec::new();
    let mut deps = Vec::new();
    let mut meta = HashMap::new();

    let mut _current_field = "";
    let mut in_desc_multiline = false;
    let mut in_meta = false;
    let mut in_deps = false;
    let mut in_tools = false;

    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'');
            _current_field = key;
            in_desc_multiline = false;

            match key {
                "name" => {
                    in_meta = false;
                    in_deps = false;
                    in_tools = false;
                    if !val.is_empty() {
                        name = val.to_string();
                    }
                }
                "description" => {
                    in_meta = false;
                    in_deps = false;
                    in_tools = false;
                    if val == ">" || val == "|" || val == ">-" || val == "|-" {
                        in_desc_multiline = true;
                    } else {
                        desc = val.to_string();
                    }
                }
                "license" => {
                    in_meta = false;
                    in_deps = false;
                    in_tools = false;
                    license = Some(val.to_string());
                }
                "compatibility" => {
                    in_meta = false;
                    in_deps = false;
                    in_tools = false;
                    comp = Some(val.to_string());
                }
                "allowed-tools" | "allowed_tools" => {
                    in_meta = false;
                    in_deps = false;
                    in_tools = true;
                    let clean = val.trim_matches('[').trim_matches(']');
                    for t in clean.split(',') {
                        let tool = t.trim().trim_matches('"').trim_matches('\'');
                        if !tool.is_empty() {
                            tools.push(tool.to_string());
                        }
                    }
                }
                "depends" | "dependencies" | "requires" => {
                    in_meta = false;
                    in_tools = false;
                    in_deps = true;
                    let clean = val.trim_matches('[').trim_matches(']');
                    for d in clean.split(',') {
                        let dep = d.trim().trim_matches('"').trim_matches('\'');
                        if !dep.is_empty() {
                            deps.push(dep.to_string());
                        }
                    }
                }
                "metadata" => {
                    in_meta = true;
                    in_deps = false;
                    in_tools = false;
                }
                _ => {
                    if in_meta {
                        meta.insert(key.to_string(), val.to_string());
                    }
                }
            }
        } else if trimmed.starts_with('-') && in_deps {
            let item = trimmed.trim_start_matches('-').trim().trim_matches('"').trim_matches('\'');
            if !item.is_empty() {
                deps.push(item.to_string());
            }
        } else if trimmed.starts_with('-') && in_tools {
            let item = trimmed.trim_start_matches('-').trim().trim_matches('"').trim_matches('\'');
            if !item.is_empty() {
                tools.push(item.to_string());
            }
        } else if in_desc_multiline {
            if !desc.is_empty() {
                desc.push(' ');
            }
            desc.push_str(trimmed);
        } else if in_meta && line.starts_with("  ") {
            if let Some((k, v)) = trimmed.split_once(':') {
                meta.insert(
                    k.trim().to_string(),
                    v.trim().trim_matches('"').trim_matches('\'').to_string(),
                );
            }
        }
    }

    (name, desc, license, comp, tools, deps, meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let raw = "---\nname: test-skill\ndescription: Test description\n---\n# Body here\nHello";
        let (fm, body) = parse_frontmatter(raw);
        assert!(fm.contains("name: test-skill"));
        assert!(body.starts_with("# Body here"));

        let (name, desc, _, _, _tools, _deps, _) = parse_yaml_metadata(&fm, "fallback");
        assert_eq!(name, "test-skill");
        assert_eq!(desc, "Test description");

        // Multi-line dependencies & tools
        let raw_lists = r#"
name: list-skill
dependencies:
  - dep-one
  - dep-two
allowed-tools:
  - Read
  - Write
"#;
        let (_, _, _, _, tools, deps, _) = parse_yaml_metadata(raw_lists, "fallback");
        assert_eq!(deps, vec!["dep-one", "dep-two"]);
        assert_eq!(tools, vec!["Read", "Write"]);
    }

    #[test]
    fn test_is_ignored_path() {
        assert!(is_ignored_path(Path::new("foo/__pycache__/bar.pyc")));
        assert!(is_ignored_path(Path::new("foo/.git/config")));
        assert!(!is_ignored_path(Path::new("references/guide/index.md")));
    }
}

