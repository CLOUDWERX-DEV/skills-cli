//! Making, installing and removing skills: `skills new`, `skills install`, `skills remove`.
use crate::config::Config;
use crate::output::{bold, cyan, dim, green, yellow};
use crate::registry::SkillRegistry;
use crate::skill::Skill;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A valid skill name: lowercase letters, digits and single hyphens, no hyphen at either end, at most 64 characters.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Where new and installed skills go: `--dir`, else the first library folder in the configuration.
fn target_dir(config: &Config, dir: Option<&Path>) -> Result<PathBuf> {
    match dir {
        Some(d) => Ok(Config::resolve_path(&d.to_string_lossy())),
        None => config.skills_dirs.first().cloned().context("no skills folder is configured: pass --dir or run `skills config init`"),
    }
}

pub fn new_skill(config: &Config, name: &str, description: Option<&str>, dir: Option<&Path>) -> Result<()> {
    if !valid_name(name) {
        bail!("'{name}' isn't a valid skill name: use lowercase letters, digits and single hyphens (at most 64 characters), like `pdf-tools`");
    }
    let root = target_dir(config, dir)?.join(name);
    if root.exists() {
        bail!("{} already exists", root.display());
    }
    fs::create_dir_all(root.join("references")).with_context(|| format!("creating {}", root.display()))?;
    fs::create_dir_all(root.join("scripts"))?;
    let description = description.map(str::to_string).unwrap_or_else(|| format!("Describe what {name} does, then say when to use it: the tasks, files or tools that should make an agent reach for it."));
    let text = format!(
        "---\nname: {name}\ndescription: {description}\nlicense: MIT\n---\n\n# {title}\n\nWhat this skill is for, in a sentence or two.\n\n## When to use it\n\n- the situations that should trigger it\n\n## How to do it\n\n1. The steps, in order.\n2. Keep this file short. Put long material in `references/` and link to it, like [the details](references/details.md).\n3. Put helper programs in `scripts/` and run them with `skills run {name} <script>`.\n",
        title = name.replace('-', " ")
    );
    fs::write(root.join("SKILL.md"), text)?;
    fs::write(root.join("references/details.md"), "# Details\n\nLonger notes that don't need to load every time.\n")?;
    println!("{} {}", green("Created"), root.display());
    println!("{}", dim("Edit SKILL.md, then check it with `skills validate` and try it with `skills load`."));
    Ok(())
}

/// Turn what the person typed into something `git clone` takes, or None when it's a local path.
pub fn git_url(source: &str) -> Option<String> {
    if Path::new(source).exists() {
        return None;
    }
    if source.starts_with("http://") || source.starts_with("https://") || source.starts_with("git@") || source.starts_with("ssh://") || source.starts_with("file://") {
        return Some(source.to_string());
    }
    let s = source.trim_end_matches('/');
    if let Some(rest) = s.strip_prefix("github.com/") {
        return Some(format!("https://github.com/{}.git", rest.trim_end_matches(".git")));
    }
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))) {
        return Some(format!("https://github.com/{}/{}.git", parts[0], parts[1].trim_end_matches(".git")));
    }
    None
}

/// The skills under `root`: the folder itself if it has a SKILL.md, else every folder below it (up to three deep) that has one.
pub fn find_skills(root: &Path) -> Vec<PathBuf> {
    if root.join("SKILL.md").is_file() || root.join("skill.md").is_file() {
        return vec![root.to_path_buf()];
    }
    let mut found = Vec::new();
    fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
        if depth == 0 {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else { return };
        let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
        dirs.sort();
        for d in dirs {
            if d.join("SKILL.md").is_file() || d.join("skill.md").is_file() {
                found.push(d);
            } else {
                walk(&d, depth - 1, found);
            }
        }
    }
    walk(root, 3, &mut found);
    found
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(&name));
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_dir(&src, &dst)?;
        } else if kind.is_symlink() {
            // follow links to files, skip links to folders (they could point anywhere, including back into the tree)
            if src.is_file() {
                fs::copy(&src, &dst)?;
            }
        } else {
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

pub fn install(config: &Config, source: &str, subdir: Option<&str>, only: &[String], dir: Option<&Path>, force: bool) -> Result<()> {
    let target = target_dir(config, dir)?;
    let mut cloned: Option<PathBuf> = None;
    let root = match git_url(source) {
        Some(url) => {
            let tmp = std::env::temp_dir().join(format!("skills-install-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
            println!("{} {}", dim("Cloning"), url);
            let status = Command::new("git").args(["clone", "--depth", "1", "--quiet", &url]).arg(&tmp).status().context("can't run git: is it installed?")?;
            if !status.success() {
                bail!("git couldn't clone {url}");
            }
            cloned = Some(tmp.clone());
            tmp
        }
        None => PathBuf::from(source),
    };
    let result = (|| -> Result<()> {
        if !root.is_dir() {
            bail!("'{source}' isn't a folder, a git URL or an owner/repo on GitHub");
        }
        let search = match subdir {
            Some(s) => {
                let p = root.join(s);
                if !p.starts_with(&root) || s.contains("..") {
                    bail!("--path must stay inside the source");
                }
                p
            }
            None => root.clone(),
        };
        let mut skills = find_skills(&search);
        if !only.is_empty() {
            skills.retain(|p| p.file_name().is_some_and(|n| only.iter().any(|o| o.as_str() == n.to_string_lossy())));
        }
        if skills.is_empty() {
            bail!("no skills found in {} (a skill is a folder with a SKILL.md)", search.display());
        }
        fs::create_dir_all(&target)?;
        let mut installed = 0;
        for path in &skills {
            let skill = Skill::from_dir(path).context("not a skill folder")?;
            let folder = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| skill.name.clone());
            let name = if path == &root { source.trim_end_matches('/').rsplit('/').next().unwrap_or(&skill.name).trim_end_matches(".git").to_string() } else { folder };
            let name = if valid_name(&name) { name } else { skill.name.clone() };
            if !valid_name(&name) {
                println!("  {} {} {}", yellow("skipped"), path.display(), dim("(no valid name)"));
                continue;
            }
            if skill.description.trim().is_empty() {
                println!("  {} {} {}", yellow("skipped"), name, dim("(no description in its frontmatter)"));
                continue;
            }
            let dest = target.join(&name);
            if dest.exists() {
                if !force {
                    println!("  {} {} {}", yellow("exists"), name, dim("(use --force to replace it)"));
                    continue;
                }
                if dest.is_symlink() {
                    fs::remove_file(&dest)?;
                } else {
                    fs::remove_dir_all(&dest)?;
                }
            }
            copy_dir(path, &dest)?;
            println!("  {} {} {}", green("installed"), bold(&name), dim(&dest.display().to_string()));
            installed += 1;
        }
        println!("\n{} skill(s) installed into {}", installed, cyan(&target.display().to_string()));
        Ok(())
    })();
    if let Some(tmp) = cloned {
        let _ = fs::remove_dir_all(tmp);
    }
    result
}

pub fn remove(registry: &SkillRegistry, names: &[String], yes: bool) -> Result<()> {
    if names.is_empty() {
        bail!("name the skill(s) to remove");
    }
    for n in names {
        let Some(skill) = registry.find(n) else { bail!("no skill called '{n}'") };
        let path = &skill.dir_path;
        if path.parent().is_none() || path.components().count() < 3 {
            bail!("refusing to remove {}", path.display());
        }
        if !yes {
            println!("{} {} ({})", yellow("Would remove"), skill.name, path.display());
            continue;
        }
        if path.is_symlink() {
            fs::remove_file(path)?; // a link: remove the link, never what it points at
        } else {
            fs::remove_dir_all(path)?;
        }
        println!("{} {}", green("Removed"), skill.name);
    }
    if !yes {
        println!("{}", dim("Nothing was removed. Add --yes to do it."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("skills-test-{tag}-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn config(dir: &Path) -> Config {
        Config { skills_dirs: vec![dir.to_path_buf()], max_ref_tokens: 30_000, config_file_path: dir.join("config.toml") }
    }

    fn make_skill(root: &Path, name: &str, description: &str) {
        fs::create_dir_all(root.join(name)).unwrap();
        fs::write(root.join(name).join("SKILL.md"), format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n")).unwrap();
    }

    #[test]
    fn names() {
        for ok in ["pdf", "pdf-tools", "a1", "x-2-y"] {
            assert!(valid_name(ok), "{ok}");
        }
        for bad in ["", "-a", "a-", "a--b", "A", "a_b", "a b", "../x", &"a".repeat(65)] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn new_makes_a_skill_that_loads_and_refuses_to_overwrite() {
        let d = tmp("new");
        new_skill(&config(&d), "my-skill", None, None).unwrap();
        let s = Skill::from_dir(&d.join("my-skill")).unwrap();
        assert_eq!(s.name, "my-skill");
        assert!(!s.description.is_empty());
        assert!(d.join("my-skill/references/details.md").is_file() && d.join("my-skill/scripts").is_dir());
        assert!(new_skill(&config(&d), "my-skill", None, None).is_err(), "never overwrites");
        assert!(new_skill(&config(&d), "Bad Name", None, None).is_err());
        new_skill(&config(&d), "described", Some("Does a thing. Use when asked."), None).unwrap();
        assert_eq!(Skill::from_dir(&d.join("described")).unwrap().description, "Does a thing. Use when asked.");
    }

    #[test]
    fn git_urls() {
        assert_eq!(git_url("owner/repo").as_deref(), Some("https://github.com/owner/repo.git"));
        assert_eq!(git_url("github.com/owner/repo").as_deref(), Some("https://github.com/owner/repo.git"));
        assert_eq!(git_url("https://example.com/x.git").as_deref(), Some("https://example.com/x.git"));
        assert_eq!(git_url("git@github.com:o/r.git").as_deref(), Some("git@github.com:o/r.git"));
        assert_eq!(git_url("/tmp"), None, "an existing path is local");
        assert_eq!(git_url("nonexistent-single-word"), None);
    }

    #[test]
    fn finds_one_skill_or_many() {
        let d = tmp("find");
        make_skill(&d, "one", "first");
        make_skill(&d.join("nested/deeper"), "two", "second");
        fs::create_dir_all(d.join(".hidden")).unwrap();
        make_skill(&d.join(".hidden"), "ghost", "never found");
        let mut found: Vec<String> = find_skills(&d).iter().map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).collect();
        found.sort();
        assert_eq!(found, vec!["one", "two"]);
        assert_eq!(find_skills(&d.join("one")), vec![d.join("one")]);
    }

    #[test]
    fn install_copies_skips_existing_and_forces() {
        let (src, lib) = (tmp("src"), tmp("lib"));
        make_skill(&src, "alpha", "Alpha skill. Use when needed.");
        make_skill(&src, "beta", "Beta skill. Use when needed.");
        fs::create_dir_all(src.join("alpha/references")).unwrap();
        fs::write(src.join("alpha/references/doc.md"), "doc").unwrap();
        fs::create_dir_all(src.join("alpha/.git")).unwrap();
        fs::write(src.join("alpha/.git/config"), "x").unwrap();
        install(&config(&lib), src.to_str().unwrap(), None, &[], None, false).unwrap();
        assert!(lib.join("alpha/references/doc.md").is_file() && lib.join("beta/SKILL.md").is_file());
        assert!(!lib.join("alpha/.git").exists(), ".git is never copied");
        fs::write(lib.join("alpha/SKILL.md"), "---\nname: alpha\ndescription: edited\n---\n").unwrap();
        install(&config(&lib), src.to_str().unwrap(), None, &[], None, false).unwrap();
        assert!(fs::read_to_string(lib.join("alpha/SKILL.md")).unwrap().contains("edited"), "an existing skill is left alone");
        install(&config(&lib), src.to_str().unwrap(), None, &[], None, true).unwrap();
        assert!(fs::read_to_string(lib.join("alpha/SKILL.md")).unwrap().contains("Alpha skill"), "--force replaces it");
    }

    #[test]
    fn install_can_pick_skills_and_refuses_nonsense() {
        let (src, lib) = (tmp("src2"), tmp("lib2"));
        make_skill(&src, "alpha", "A. Use when.");
        make_skill(&src, "beta", "B. Use when.");
        install(&config(&lib), src.to_str().unwrap(), None, &["beta".into()], None, false).unwrap();
        assert!(lib.join("beta").is_dir() && !lib.join("alpha").exists());
        assert!(install(&config(&lib), "/definitely/not/here", None, &[], None, false).is_err());
        assert!(install(&config(&lib), src.to_str().unwrap(), Some("../.."), &[], None, false).is_err(), "--path can't leave the source");
        let empty = tmp("empty");
        assert!(install(&config(&lib), empty.to_str().unwrap(), None, &[], None, false).is_err());
    }

    #[test]
    fn install_skips_a_skill_with_no_description() {
        let (src, lib) = (tmp("src3"), tmp("lib3"));
        fs::create_dir_all(src.join("bare")).unwrap();
        fs::write(src.join("bare/SKILL.md"), "---\nname: bare\n---\n# bare\n").unwrap();
        install(&config(&lib), src.to_str().unwrap(), None, &[], None, false).unwrap();
        assert!(!lib.join("bare").exists());
    }

    #[test]
    fn remove_needs_yes_and_never_follows_a_link() {
        let (lib, other) = (tmp("rm"), tmp("rm-other"));
        make_skill(&lib, "gone", "Goes. Use when.");
        make_skill(&other, "keep", "Stays. Use when.");
        std::os::unix::fs::symlink(other.join("keep"), lib.join("linked")).unwrap();
        let reg = SkillRegistry::discover(&config(&lib));
        remove(&reg, &["gone".into()], false).unwrap();
        assert!(lib.join("gone").exists(), "without --yes nothing is removed");
        remove(&reg, &["gone".into()], true).unwrap();
        assert!(!lib.join("gone").exists());
        if let Some(l) = reg.find("keep") {
            let _ = l;
        }
        let reg = SkillRegistry::discover(&config(&lib));
        if reg.find("keep").is_some() {
            remove(&reg, &["keep".into()], true).unwrap();
            assert!(other.join("keep/SKILL.md").exists(), "the target of a link survives");
        }
        assert!(remove(&reg, &["nope".into()], true).is_err());
    }
}
