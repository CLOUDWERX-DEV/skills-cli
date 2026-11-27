use crate::agents::AgentManager;
use crate::output::{bold, cyan, dim, green, red, yellow};
use crate::registry::SkillRegistry;
use anyhow::{bail, Result};

pub fn execute_pin(
    registry: &SkillRegistry,
    skills: &[String],
    agents_filter: Option<&str>,
) -> Result<()> {
    if skills.is_empty() {
        bail!("No skill names provided to pin. Usage: skills pin <skill1> [skill2 ...] [--agents <list>]");
    }

    let agents = AgentManager::resolve_agents(agents_filter);
    if agents.is_empty() {
        println!("No supported agents detected or matched.");
        return Ok(());
    }

    let mut found_skills = Vec::new();
    for name in skills {
        if let Some(s) = registry.find(name) {
            found_skills.push(s);
        } else {
            eprintln!("{} Skill '{}' not found in vault.", yellow("Warning:"), name);
        }
    }

    if found_skills.is_empty() {
        bail!("None of the specified skills were found in the vault.");
    }

    println!(
        "Pinning {} skill(s) into {} agent(s):\n",
        bold(&found_skills.len().to_string()),
        bold(&agents.len().to_string())
    );

    for agent in &agents {
        println!("{} ({})", bold(agent.display_name()), dim(agent.id()));
        for skill in &found_skills {
            match AgentManager::pin(*agent, &skill.name, &skill.dir_path) {
                Ok(paths) => {
                    for p in paths {
                        println!("  {} {} -> {}", green("✓ Pinned:"), cyan(&skill.name), p.display());
                    }
                }
                Err(e) => {
                    println!("  {} Failed pinning {}: {e}", red("✗ Error:"), skill.name);
                }
            }
        }
        println!();
    }

    println!(
        "Tip: Pinned skills will now appear directly in your agent's native slash-command menu (e.g. /{} in Claude/Codex).",
        found_skills[0].name
    );
    println!("Run '{}' to reset back to a lean on-demand context.", cyan("skills unpin --all"));

    Ok(())
}

pub fn execute_unpin(
    skills: &[String],
    unpin_all: bool,
    agents_filter: Option<&str>,
) -> Result<()> {
    if skills.is_empty() && !unpin_all {
        bail!("Specify skill(s) to unpin or use --all. Usage: skills unpin <skill...> | skills unpin --all");
    }

    let agents = AgentManager::resolve_agents(agents_filter);
    if agents.is_empty() {
        println!("No supported agents detected or matched.");
        return Ok(());
    }

    if unpin_all {
        println!("Unpinning ALL non-system skills from {} agent(s):\n", agents.len());
        for agent in &agents {
            match AgentManager::unpin_all(*agent) {
                Ok(count) => {
                    println!(
                        "  {} {}: removed {} pinned skill symlink(s)",
                        green("✓"),
                        bold(agent.display_name()),
                        count
                    );
                }
                Err(e) => {
                    println!("  {} {}: {e}", red("✗"), agent.display_name());
                }
            }
        }
    } else {
        println!("Unpinning {} skill(s) from {} agent(s):\n", skills.len(), agents.len());
        for agent in &agents {
            for skill_name in skills {
                match AgentManager::unpin(*agent, skill_name) {
                    Ok(count) if count > 0 => {
                        println!(
                            "  {} Unpinned '{}' from {}",
                            green("✓"),
                            cyan(skill_name),
                            agent.display_name()
                        );
                    }
                    Ok(_) => {
                        println!(
                            "  {} '{}' was not pinned in {}",
                            dim("•"),
                            skill_name,
                            agent.display_name()
                        );
                    }
                    Err(e) => {
                        println!("  {} Failed unpinning {}: {e}", red("✗"), skill_name);
                    }
                }
            }
        }
    }

    println!("\nContext cleared. Only 'skills-cli' and system skills remain active.");
    Ok(())
}

pub fn execute_pinned(agents_filter: Option<&str>) -> Result<()> {
    let agents = AgentManager::resolve_agents(agents_filter);
    if agents.is_empty() {
        println!("No supported agents detected.");
        return Ok(());
    }

    println!("{}", bold(&cyan("Currently Pinned Skills by Agent:")));
    println!("{}", dim(&"─".repeat(70)));

    let mut total_pinned = 0;

    for agent in &agents {
        let items = AgentManager::list_pinned(*agent);
        println!(
            "\n{} ({}) — {} pinned:",
            bold(agent.display_name()),
            dim(agent.id()),
            items.len()
        );

        if items.is_empty() {
            println!("  {}", dim("No skills pinned (lean context mode)."));
        } else {
            for (name, path, is_link) in &items {
                total_pinned += 1;
                let target = if *is_link {
                    match std::fs::read_link(path) {
                        Ok(t) => {
                            if t.exists() {
                                format!("-> {}", t.display())
                            } else {
                                format!("-> {} {}", t.display(), red("[BROKEN LINK]"))
                            }
                        }
                        Err(_) => "[unreadable link]".to_string(),
                    }
                } else {
                    yellow("[REAL FOLDER - not a symlink!]")
                };

                println!("  • {:<24} {}", cyan(name), dim(&target));
            }
        }
    }

    println!("\n{}", dim(&"─".repeat(70)));
    println!(
        "Total pinned skills across agents: {}. Use '{}' to remove.",
        bold(&total_pinned.to_string()),
        cyan("skills unpin <name> | skills unpin --all")
    );

    Ok(())
}
