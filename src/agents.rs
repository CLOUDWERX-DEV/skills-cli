use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentKind {
    Claude,
    Codex,
    Agy,
    Agents,
    OpenCode,
    Pi,
    Hermes,
}

impl AgentKind {
    pub fn all() -> &'static [AgentKind] {
        &[
            AgentKind::Claude,
            AgentKind::Codex,
            AgentKind::Agy,
            AgentKind::Agents,
            AgentKind::OpenCode,
            AgentKind::Pi,
            AgentKind::Hermes,
        ]
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "claude" | "claude-code" => Some(AgentKind::Claude),
            "codex" | "openai-codex" => Some(AgentKind::Codex),
            "agy" | "antigravity" | "gemini" => Some(AgentKind::Agy),
            "agents" | ".agents" => Some(AgentKind::Agents),
            "opencode" => Some(AgentKind::OpenCode),
            "pi" => Some(AgentKind::Pi),
            "hermes" => Some(AgentKind::Hermes),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            AgentKind::Claude => "claude",
            AgentKind::Codex => "codex",
            AgentKind::Agy => "agy",
            AgentKind::Agents => "agents",
            AgentKind::OpenCode => "opencode",
            AgentKind::Pi => "pi",
            AgentKind::Hermes => "hermes",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentKind::Claude => "Claude Code",
            AgentKind::Codex => "OpenAI Codex",
            AgentKind::Agy => "Google Antigravity",
            AgentKind::Agents => ".agents (Standard)",
            AgentKind::OpenCode => "OpenCode",
            AgentKind::Pi => "Pi Agent",
            AgentKind::Hermes => "Hermes Agent",
        }
    }

    pub fn skill_dirs(&self) -> Vec<PathBuf> {
        let home = match std::env::var_os("HOME") {
            Some(h) => PathBuf::from(h),
            None => return Vec::new(),
        };

        match self {
            AgentKind::Claude => vec![home.join(".claude").join("skills")],
            AgentKind::Codex => vec![home.join(".codex").join("skills")],
            AgentKind::Agy => vec![
                home.join(".gemini").join("antigravity-cli").join("skills"),
                home.join(".gemini").join("config").join("skills"),
            ],
            AgentKind::Agents => vec![home.join(".agents").join("skills")],
            AgentKind::OpenCode => vec![
                home.join(".config").join("opencode").join("skills"),
                home.join(".opencode").join("skills"),
            ],
            AgentKind::Pi => vec![
                home.join(".pi").join("agent").join("skills"),
                home.join(".pi").join("skills"),
            ],
            AgentKind::Hermes => vec![home.join(".hermes").join("skills")],
        }
    }

    pub fn is_installed(&self) -> bool {
        let home = match std::env::var_os("HOME") {
            Some(h) => PathBuf::from(h),
            None => return false,
        };

        match self {
            AgentKind::Claude => home.join(".claude").exists(),
            AgentKind::Codex => home.join(".codex").exists(),
            AgentKind::Agy => home.join(".gemini").exists(),
            AgentKind::Agents => home.join(".agents").exists(),
            AgentKind::OpenCode => {
                home.join(".config").join("opencode").exists() || home.join(".opencode").exists()
            }
            AgentKind::Pi => home.join(".pi").exists(),
            AgentKind::Hermes => home.join(".hermes").exists(),
        }
    }

    pub fn is_system_item(&self, name: &str) -> bool {
        if name == "skills-cli" || name == "synced" || name.starts_with('.') {
            return true;
        }
        if *self == AgentKind::Codex && name == ".system" {
            return true;
        }
        false
    }
}

pub struct AgentManager;

impl AgentManager {
    pub fn detected_agents() -> Vec<AgentKind> {
        AgentKind::all()
            .iter()
            .copied()
            .filter(|a| a.is_installed())
            .collect()
    }

    pub fn resolve_agents(input: Option<&str>) -> Vec<AgentKind> {
        if let Some(list_str) = input {
            let mut result = Vec::new();
            for part in list_str.split(',') {
                let clean = part.trim();
                if clean == "all" {
                    return AgentKind::all().to_vec();
                }
                if let Some(kind) = AgentKind::parse(clean) {
                    if !result.contains(&kind) {
                        result.push(kind);
                    }
                }
            }
            result
        } else {
            Self::detected_agents()
        }
    }

    pub fn pin(agent: AgentKind, skill_name: &str, target_path: &Path) -> Result<Vec<PathBuf>> {
        let mut pinned = Vec::new();
        for dir in agent.skill_dirs() {
            if crate::config::is_library_folder(&dir) {
                continue; // the library itself through a link (~/.agents/skills): never pin into or unpin from it
            }
            if let Some(parent) = dir.parent() {
                if !parent.exists() {
                    continue; // agent parent root doesn't exist on this machine
                }
            }
            fs::create_dir_all(&dir)
                .with_context(|| format!("Creating skills dir {}", dir.display()))?;

            let link_path = dir.join(skill_name);
            if link_path.is_symlink() || link_path.exists() {
                // If it's a symlink, remove it first
                if link_path.is_symlink() {
                    let _ = fs::remove_file(&link_path);
                } else if link_path.is_dir() {
                    // Safety: do not clobber a real directory silently
                    continue;
                }
            }

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(target_path, &link_path)
                    .with_context(|| format!("Symlinking {} -> {}", link_path.display(), target_path.display()))?;
                pinned.push(link_path);
            }
        }
        Ok(pinned)
    }

    pub fn unpin(agent: AgentKind, skill_name: &str) -> Result<usize> {
        let mut removed = 0;
        for dir in agent.skill_dirs() {
            if crate::config::is_library_folder(&dir) {
                continue; // the library itself through a link (~/.agents/skills): never pin into or unpin from it
            }
            if !dir.is_dir() {
                continue;
            }
            let link_path = dir.join(skill_name);
            if link_path.is_symlink() {
                fs::remove_file(&link_path)?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn unpin_all(agent: AgentKind) -> Result<usize> {
        let mut removed = 0;
        for dir in agent.skill_dirs() {
            if crate::config::is_library_folder(&dir) {
                continue; // the library itself through a link (~/.agents/skills): never pin into or unpin from it
            }
            if !dir.is_dir() {
                continue;
            }
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };
                if agent.is_system_item(name) {
                    continue;
                }
                if path.is_symlink() {
                    fs::remove_file(&path)?;
                    removed += 1;
                }
            }
        }
        Ok(removed)
    }

    pub fn list_pinned(agent: AgentKind) -> Vec<(String, PathBuf, bool)> {
        let mut results = Vec::new();
        for dir in agent.skill_dirs() {
            if crate::config::is_library_folder(&dir) {
                continue; // the library itself through a link (~/.agents/skills): never pin into or unpin from it
            }
            if !dir.is_dir() {
                continue;
            }
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if agent.is_system_item(&name) {
                    continue;
                }
                let is_link = path.is_symlink();
                results.push((name, path, is_link));
            }
        }
        results
    }
}
