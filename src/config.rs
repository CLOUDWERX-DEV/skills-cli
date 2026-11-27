use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfigFile {
    #[serde(default)]
    pub skills_dirs: Vec<String>,
    #[serde(default)]
    pub max_ref_tokens: Option<usize>,
}

impl Default for SkillsConfigFile {
    fn default() -> Self {
        Self {
            skills_dirs: vec![
                "~/.agents/skills".to_string(),
                "~/.claude/skills".to_string(),
                "~/.codex/skills".to_string(),
                "~/.gemini/antigravity-cli/skills".to_string(),
            ],
            max_ref_tokens: Some(30_000),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub skills_dirs: Vec<PathBuf>,
    pub max_ref_tokens: usize,
    pub config_file_path: PathBuf,
}

impl Config {
    pub fn resolve_path(p: &str) -> PathBuf {
        if let Some(stripped) = p.strip_prefix("~/") {
            if let Some(home) = dirs_home() {
                return home.join(stripped);
            }
        } else if p == "~" {
            if let Some(home) = dirs_home() {
                return home;
            }
        }
        PathBuf::from(p)
    }

    pub fn default_config_path() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("skills-cli").join("config.toml");
            }
        }
        if let Some(home) = dirs_home() {
            home.join(".config").join("skills-cli").join("config.toml")
        } else {
            PathBuf::from("skills-config.toml")
        }
    }

    pub fn load(cli_dirs: &[PathBuf], custom_config: Option<&Path>) -> Result<Self> {
        let config_file_path = match custom_config {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path(),
        };

        let file_cfg = if config_file_path.exists() {
            let content = std::fs::read_to_string(&config_file_path)
                .with_context(|| format!("Reading config file {}", config_file_path.display()))?;
            toml::from_str::<SkillsConfigFile>(&content)
                .with_context(|| format!("Parsing config file {}", config_file_path.display()))?
        } else {
            SkillsConfigFile::default()
        };

        let mut dirs = Vec::new();

        // 1. CLI explicit directories take highest priority
        for d in cli_dirs {
            let resolved = Self::resolve_path(&d.to_string_lossy());
            if !dirs.contains(&resolved) {
                dirs.push(resolved);
            }
        }

        // 2. SKILLS_PATH environment variable (colon-separated)
        if let Ok(env_val) = std::env::var("SKILLS_PATH") {
            for part in std::env::split_paths(&env_val) {
                let resolved = Self::resolve_path(&part.to_string_lossy());
                if !dirs.contains(&resolved) {
                    dirs.push(resolved);
                }
            }
        }

        // 3. Project-local directories if they exist in CWD (.agents/skills, .claude/skills)
        let cwd = std::env::current_dir().unwrap_or_default();
        let local_agents = cwd.join(".agents").join("skills");
        if local_agents.is_dir() && !dirs.contains(&local_agents) {
            dirs.push(local_agents);
        }
        let local_claude = cwd.join(".claude").join("skills");
        if local_claude.is_dir() && !dirs.contains(&local_claude) {
            dirs.push(local_claude);
        }

        // 4. Config file / default directories
        for entry in &file_cfg.skills_dirs {
            let resolved = Self::resolve_path(entry);
            if !dirs.contains(&resolved) {
                dirs.push(resolved);
            }
        }

        let max_ref_tokens = file_cfg.max_ref_tokens.unwrap_or(30_000);

        Ok(Self {
            skills_dirs: dirs,
            max_ref_tokens,
            config_file_path,
        })
    }

    pub fn save_default(&self) -> Result<()> {
        if let Some(parent) = self.config_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file_cfg = SkillsConfigFile {
            skills_dirs: self
                .skills_dirs
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            max_ref_tokens: Some(self.max_ref_tokens),
        };
        let s = toml::to_string_pretty(&file_cfg)?;
        std::fs::write(&self.config_file_path, s)?;
        Ok(())
    }

    pub fn add_dir(&mut self, path: &Path) -> Result<()> {
        let abs = if path.is_relative() {
            std::env::current_dir()?.join(path)
        } else {
            path.to_path_buf()
        };
        if !self.skills_dirs.contains(&abs) {
            self.skills_dirs.push(abs);
            self.save_default()?;
        }
        Ok(())
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// True when an agent's skill folder is another configured library folder reached through a link
/// (`~/.agents` linked to a checkout, so `~/.agents/skills` is the library). Its folders are the skills
/// themselves, and linking `skills-cli` into it would replace the real skill with a link to itself.
pub fn is_library_alias(dir: &Path, libraries: &[PathBuf]) -> bool {
    let Ok(real) = std::fs::canonicalize(dir) else { return false };
    libraries
        .iter()
        .any(|lib| lib.as_path() != dir && std::fs::canonicalize(lib).map(|l| l == real).unwrap_or(false))
}

/// Is an agent skill folder really the library (see `is_library_alias`), going by the configured library folders?
pub fn is_library_folder(dir: &Path) -> bool {
    Config::load(&[], None).map(|c| is_library_alias(dir, &c.skills_dirs)).unwrap_or(false)
}
