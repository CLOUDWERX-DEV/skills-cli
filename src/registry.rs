use crate::config::Config;
use crate::skill::Skill;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub struct SkillRegistry {
    skills: Vec<Skill>,
    by_name: HashMap<String, usize>,
    search_dirs: Vec<PathBuf>,
}

impl SkillRegistry {
    pub fn discover(config: &Config) -> Self {
        let mut skills = Vec::new();
        let mut by_name = HashMap::new();
        let mut seen_canonical = HashSet::new();

        for search_dir in &config.skills_dirs {
            if !search_dir.is_dir() {
                continue;
            }

            let entries = match fs::read_dir(search_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                // Check if it is a directory or symlink pointing to a directory
                let is_dir = path.is_dir();
                if !is_dir {
                    continue;
                }
                // Hidden folders (an installer's `.system`) and folders with no SKILL.md aren't skills.
                let hidden = path.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'));
                if hidden || !(path.join("SKILL.md").is_file() || path.join("skill.md").is_file()) {
                    continue;
                }

                // Canonicalize to avoid indexing identical physical folders twice if symlinked
                let canonical = match fs::canonicalize(&path) {
                    Ok(c) => c,
                    Err(_) => path.clone(),
                };

                if seen_canonical.contains(&canonical) {
                    continue;
                }

                if let Some(skill) = Skill::from_dir(&path) {
                    let key = skill.name.to_lowercase();
                    if let std::collections::hash_map::Entry::Vacant(slot) = by_name.entry(key) {
                        seen_canonical.insert(canonical);
                        slot.insert(skills.len());
                        skills.push(skill);
                    }
                }
            }
        }

        // Sort skills alphabetically by name
        skills.sort_by_key(|s| s.name.to_lowercase());
        by_name.clear();
        for (i, s) in skills.iter().enumerate() {
            by_name.insert(s.name.to_lowercase(), i);
        }

        Self {
            skills,
            by_name,
            search_dirs: config.skills_dirs.clone(),
        }
    }

    pub fn all(&self) -> &[Skill] {
        &self.skills
    }

    pub fn search_dirs(&self) -> &[PathBuf] {
        &self.search_dirs
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        let key = name.trim().to_lowercase();
        self.by_name.get(&key).map(|&idx| &self.skills[idx])
    }

    pub fn find(&self, query: &str) -> Option<&Skill> {
        let clean = query.trim().to_lowercase();
        // 1. Exact match
        if let Some(s) = self.get(&clean) {
            return Some(s);
        }

        // 2. Exact match replacing hyphens with underscores or vice versa
        let swapped = clean.replace('_', "-");
        if let Some(s) = self.get(&swapped) {
            return Some(s);
        }
        let swapped_under = clean.replace('-', "_");
        if let Some(s) = self.get(&swapped_under) {
            return Some(s);
        }

        // 3. Prefix match
        for s in &self.skills {
            let sname = s.name.to_lowercase();
            if sname.starts_with(&clean) {
                return Some(s);
            }
        }

        // 4. Substring match
        for s in &self.skills {
            let sname = s.name.to_lowercase();
            if sname.contains(&clean) {
                return Some(s);
            }
        }

        None
    }

    pub fn search(&self, query: &str) -> Vec<(&Skill, usize)> {
        let q = query.trim().to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();
        let mut results = Vec::new();

        for skill in &self.skills {
            let name = skill.name.to_lowercase();
            let desc = skill.description.to_lowercase();
            let comp = skill.compatibility.as_deref().unwrap_or("").to_lowercase();

            let mut score = 0;

            // Exact name match gets highest score
            if name == q {
                score += 1000;
            } else if name.starts_with(&q) {
                score += 500;
            } else if name.contains(&q) {
                score += 250;
            }

            // Word-level matching
            let mut all_terms_match = true;
            for term in &terms {
                let mut term_score = 0;
                if name.contains(term) {
                    term_score += 100;
                }
                if desc.contains(term) {
                    term_score += 30;
                }
                if comp.contains(term) {
                    term_score += 10;
                }

                if term_score == 0 {
                    all_terms_match = false;
                } else {
                    score += term_score;
                }
            }

            if all_terms_match || score > 50 {
                results.push((skill, score));
            }
        }

        results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));
        results
    }

    pub fn global_stats(&self) -> GlobalStats {
        let mut total_files = 0;
        let mut total_bytes = 0;
        let mut total_tokens = 0;
        let mut total_refs = 0;
        let mut total_ref_bytes = 0;
        let mut total_ref_tokens = 0;
        let mut total_scripts = 0;
        let mut total_assets = 0;
        let mut total_md_bytes = 0;
        let mut total_md_tokens = 0;

        for s in &self.skills {
            let st = s.stats();
            total_files += st.total_files;
            total_bytes += st.total_bytes;
            total_tokens += st.total_tokens;
            total_refs += st.ref_count;
            total_ref_bytes += st.ref_bytes;
            total_ref_tokens += st.ref_tokens;
            total_scripts += st.script_count;
            total_assets += st.asset_count;
            total_md_bytes += st.skill_md_bytes;
            total_md_tokens += st.skill_md_tokens;
        }

        GlobalStats {
            total_skills: self.skills.len(),
            total_files,
            total_bytes,
            total_tokens,
            total_refs,
            total_ref_bytes,
            total_ref_tokens,
            total_scripts,
            total_assets,
            total_md_bytes,
            total_md_tokens,
        }
    }
}

pub struct GlobalStats {
    pub total_skills: usize,
    pub total_files: usize,
    pub total_bytes: u64,
    pub total_tokens: usize,
    pub total_refs: usize,
    pub total_ref_bytes: u64,
    pub total_ref_tokens: usize,
    pub total_scripts: usize,
    pub total_assets: usize,
    pub total_md_bytes: u64,
    pub total_md_tokens: usize,
}
