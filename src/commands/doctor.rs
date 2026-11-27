use crate::agents::AgentKind;
use crate::config::Config;
use crate::output::{bold, cyan, dim, green, red, yellow};
use crate::registry::SkillRegistry;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct DoctorIssue {
    pub category: &'static str,
    pub description: String,
    pub fixable: bool,
    pub fix_action: Option<FixAction>,
}

pub enum FixAction {
    RemovePath(PathBuf),
    LinkSkillsCli(PathBuf),
    ChmodExecutable(PathBuf),
}

pub fn execute(config: &Config, registry: &SkillRegistry, fix: bool) -> Result<()> {
    println!("{}", bold(&cyan("══════════ Skills Doctor Health Diagnostic ══════════\n")));

    let mut issues = Vec::new();

    // 1. Audit Agent Directories
    println!("{}", bold("1. Auditing Agent Roots:"));
    for agent in AgentKind::all() {
        if !agent.is_installed() {
            println!("  {} {:<22} {}", dim("•"), agent.display_name(), dim("(not installed)"));
            continue;
        }

        let mut agent_issues = 0;
        println!("  {} {:<22} {}", green("✓"), bold(agent.display_name()), dim("installed"));

        for dir in agent.skill_dirs() {
            if !dir.is_dir() {
                continue;
            }
            if crate::config::is_library_alias(&dir, &config.skills_dirs) {
                println!(
                    "    {}",
                    dim(&format!("{} is the skill library itself (a linked path); its folders are skills, not copies", dir.display()))
                );
                continue;
            }

            // Check if skills-cli exists in this agent
            let cli_link = dir.join("skills-cli");
            if !cli_link.exists() {
                issues.push(DoctorIssue {
                    category: "Agent Setup",
                    description: format!(
                        "Missing 'skills-cli' meta-skill in {} ({})",
                        agent.display_name(),
                        dir.display()
                    ),
                    fixable: true,
                    fix_action: Some(FixAction::LinkSkillsCli(cli_link)),
                });
                agent_issues += 1;
            }

            // Inspect entries in agent skills directory
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

                // System names (skills-cli, synced) are skipped, but a dead link is reported whatever its name:
                // a dead skills-cli or a leftover `synced` link would otherwise hide here forever.
                let dead_link = path.is_symlink() && !path.exists();
                if agent.is_system_item(name) && !dead_link {
                    continue;
                }

                if path.is_symlink() {
                    match fs::read_link(&path) {
                        Ok(target) => {
                            if !target.exists() {
                                issues.push(DoctorIssue {
                                    category: "Broken Symlink",
                                    description: format!(
                                        "Dead link in {}: {} -> {} (target does not exist)",
                                        agent.display_name(),
                                        path.display(),
                                        target.display()
                                    ),
                                    fixable: true,
                                    fix_action: Some(FixAction::RemovePath(path)),
                                });
                                agent_issues += 1;
                            }
                        }
                        Err(_) => {
                            issues.push(DoctorIssue {
                                category: "Broken Symlink",
                                description: format!("Unreadable link: {}", path.display()),
                                fixable: true,
                                fix_action: Some(FixAction::RemovePath(path)),
                            });
                            agent_issues += 1;
                        }
                    }
                } else if path.is_dir() {
                    issues.push(DoctorIssue {
                        category: "Rogue Non-Symlink",
                        description: format!(
                            "Physical directory '{}' found in {} (should be in vault and symlinked, or unpinned)",
                            name,
                            dir.display()
                        ),
                        fixable: false,
                        fix_action: None,
                    });
                    agent_issues += 1;
                }
            }
        }

        if agent_issues == 0 {
            println!("    {}", dim("No issues detected in skills directories."));
        }
    }

    println!();
    println!("{}", bold("2. Auditing Skill Vault:"));
    for dir in &config.skills_dirs {
        if !dir.is_dir() {
            issues.push(DoctorIssue {
                category: "Vault Configuration",
                description: format!("Configured search directory does not exist: {}", dir.display()),
                fixable: false,
                fix_action: None,
            });
            continue;
        }

        println!("  {} Checking vault: {}", cyan("•"), dir.display());

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let skill_md = path.join("SKILL.md");
            let skill_md_lower = path.join("skill.md");

            if !skill_md.exists() && !skill_md_lower.exists() {
                issues.push(DoctorIssue {
                    category: "Missing SKILL.md",
                    description: format!("Directory in vault without SKILL.md: {}", path.display()),
                    fixable: false,
                    fix_action: None,
                });
                continue;
            }

            // Check skill details
            if let Some(skill) = registry.get(&name) {
                if skill.description.trim().is_empty() {
                    issues.push(DoctorIssue {
                        category: "Empty Description",
                        description: format!("Skill '{}' has no description in frontmatter (agents cannot auto-trigger)", skill.name),
                        fixable: false,
                        fix_action: None,
                    });
                } else if skill.description.len() > 1024 {
                    issues.push(DoctorIssue {
                        category: "Oversized Description",
                        description: format!("Skill '{}' description is {} chars (standard limit is 1,024)", skill.name, skill.description.len()),
                        fixable: false,
                        fix_action: None,
                    });
                }

                // Check script executable permissions
                for script in skill.list_scripts() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = fs::metadata(&script.path) {
                            let mode = meta.permissions().mode();
                            let is_exec = mode & 0o111 != 0;
                            let ext = script.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if !is_exec && (ext == "sh" || ext == "bash" || ext == "py" || ext == "fish") {
                                issues.push(DoctorIssue {
                                    category: "Script Permission",
                                    description: format!("Script '{}/scripts/{}' is not executable", skill.name, script.name),
                                    fixable: true,
                                    fix_action: Some(FixAction::ChmodExecutable(script.path.clone())),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    println!();
    println!("{}", dim(&"─".repeat(70)));

    if issues.is_empty() {
        println!("{}", bold(&green("✓ Everything looks healthy! No issues detected.")));
        return Ok(());
    }

    println!(
        "{} Found {} issue(s):\n",
        bold(&yellow("Diagnostic Results:")),
        issues.len()
    );

    for (i, issue) in issues.iter().enumerate() {
        let tag = match issue.category {
            "Broken Symlink" | "Missing SKILL.md" => red(&format!("[{}]", issue.category)),
            _ => yellow(&format!("[{}]", issue.category)),
        };
        let fix_tag = if issue.fixable {
            green("(auto-fixable)")
        } else {
            dim("(manual review)")
        };

        println!("  {}. {} {} {}", i + 1, tag, issue.description, fix_tag);
    }

    if fix {
        println!("\n{}", bold(&green("Applying Automated Fixes:")));
        // The meta-skill the agents get linked to. If the library has none yet, write the one built into this binary.
        let target_cli_path = match registry.get("skills-cli") {
            Some(s) => s.dir_path.clone(),
            None => {
                let base = config.skills_dirs.first().cloned().unwrap_or_else(|| PathBuf::from("."));
                let dir = base.join("skills-cli");
                if fs::create_dir_all(&dir).and_then(|_| fs::write(dir.join("SKILL.md"), crate::commands::skill_cmd::BUILTIN_SKILL_MD)).is_ok() {
                    println!("  {} Wrote the built-in skills-cli skill to {}", green("✓"), dir.display());
                }
                dir
            }
        };

        let mut fixed_count = 0;
        for issue in &issues {
            if let Some(ref action) = issue.fix_action {
                match action {
                    FixAction::RemovePath(p) => {
                        if p.is_symlink() || p.is_file() {
                            let _ = fs::remove_file(p);
                            println!("  {} Removed dead link: {}", green("✓"), p.display());
                            fixed_count += 1;
                        }
                    }
                    FixAction::LinkSkillsCli(p) => {
                        if let Some(parent) = p.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        #[cfg(unix)]
                        {
                            if let Ok(()) = std::os::unix::fs::symlink(&target_cli_path, p) {
                                println!("  {} Linked skills-cli: {}", green("✓"), p.display());
                                fixed_count += 1;
                            }
                        }
                    }
                    FixAction::ChmodExecutable(p) => {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(meta) = fs::metadata(p) {
                                let mut perms = meta.permissions();
                                perms.set_mode(perms.mode() | 0o755);
                                if let Ok(()) = fs::set_permissions(p, perms) {
                                    println!("  {} Made executable: {}", green("✓"), p.display());
                                    fixed_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("\n{} Applied {} fix(es) successfully.", green("Success:"), fixed_count);
    } else {
        let fixable_count = issues.iter().filter(|i| i.fixable).count();
        if fixable_count > 0 {
            println!(
                "\nRun '{}' to automatically resolve {} issue(s).",
                green("skills doctor --fix"),
                fixable_count
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_linked_path_to_another_library_is_the_library() {
        let base = std::env::temp_dir().join(format!("skills-doctor-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let library = base.join("repo/agents/skills");
        let own = base.join("claude/skills");
        fs::create_dir_all(&library).unwrap();
        fs::create_dir_all(&own).unwrap();
        std::os::unix::fs::symlink(base.join("repo/agents"), base.join("dot-agents")).unwrap();
        let alias = base.join("dot-agents/skills");
        let libraries = vec![library.clone(), alias.clone(), own.clone()];
        assert!(crate::config::is_library_alias(&alias, &libraries), "~/.agents/skills reached through a link is the library");
        assert!(!crate::config::is_library_alias(&own, &libraries), "an agent's own folder is audited");
        assert!(!crate::config::is_library_alias(&base.join("missing"), &libraries));
        fs::remove_dir_all(&base).ok();
    }
}
