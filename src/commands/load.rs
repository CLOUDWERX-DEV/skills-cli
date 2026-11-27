use crate::output::{cyan, estimate_tokens, format_tokens, yellow};
use crate::registry::SkillRegistry;
use anyhow::{bail, Result};

use std::collections::HashSet;

pub fn execute(
    registry: &SkillRegistry,
    names: &[String],
    raw: bool,
    with_frontmatter: bool,
    quiet: bool,
    load_deps: bool,
) -> Result<()> {
    if names.is_empty() {
        bail!("No skill names provided. Usage: skills load <skill1> [skill2 ...] [--deps]");
    }

    let mut found_skills = Vec::new();
    let mut missing = Vec::new();
    let mut seen = HashSet::new();

    for name in names {
        if let Some(skill) = registry.find(name) {
            collect_skills(registry, skill, &mut found_skills, &mut seen, load_deps);
        } else {
            missing.push(name.clone());
        }
    }

    if !missing.is_empty() {
        eprintln!(
            "{} Could not find skill(s): {}",
            yellow("Warning:"),
            missing.join(", ")
        );
        eprintln!("Run '{}' to search available skills.\n", cyan("skills search <term>"));
    }

    if found_skills.is_empty() {
        bail!("None of the specified skills were found.");
    }

    let total_tokens: usize = found_skills
        .iter()
        .map(|s| {
            if with_frontmatter {
                s.skill_md_path
                    .as_ref()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|c| estimate_tokens(&c))
                    .unwrap_or(0)
            } else {
                estimate_tokens(&s.body)
            }
        })
        .sum();

    // If multiple skills and not quiet/raw, output a top summary banner
    if !raw && !quiet && found_skills.len() > 1 {
        println!("<!-- ===================================================================== -->");
        println!("<!-- LOADED {} SKILLS BATCH | Total tokens: {:<28} -->", found_skills.len(), format_tokens(total_tokens));
        for s in &found_skills {
            let tok = estimate_tokens(&s.body);
            println!("<!--   * {:<24} ({}) {:<33} -->", s.name, format_tokens(tok), "");
        }
        println!("<!-- ===================================================================== -->\n");
    }

    for (i, skill) in found_skills.iter().enumerate() {
        let content = if with_frontmatter {
            if let Some(ref p) = skill.skill_md_path {
                std::fs::read_to_string(p).unwrap_or_else(|_| skill.body.clone())
            } else {
                skill.body.clone()
            }
        } else {
            skill.body.clone()
        };

        let tok = estimate_tokens(&content);

        if !raw && !quiet {
            println!("<!-- ===================================================================== -->");
            println!("<!-- SKILL: {} | Tokens: {} | Path: {} -->", skill.name, format_tokens(tok), skill.dir_path.display());
            if !skill.description.is_empty() {
                println!("<!-- Description: -->");
                for line in skill.description.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        println!("<!--   {} -->", trimmed);
                    }
                }
            }
            let st = skill.stats();
            if st.ref_count > 0 || st.script_count > 0 {
                println!(
                    "<!-- Auxiliary: {} references (fetch with 'skills ref {} <file>') | {} scripts -->",
                    st.ref_count, skill.name, st.script_count
                );
            }
            println!("<!-- ===================================================================== -->\n");
        }

        println!("{}", content.trim());

        if i + 1 < found_skills.len() {
            println!("\n\n");
        }
    }

    Ok(())
}

fn collect_skills<'a>(
    registry: &'a SkillRegistry,
    skill: &'a crate::skill::Skill,
    list: &mut Vec<&'a crate::skill::Skill>,
    seen: &mut HashSet<String>,
    load_deps: bool,
) {
    let key = skill.name.to_lowercase();
    if seen.contains(&key) {
        return;
    }
    seen.insert(key);
    list.push(skill);

    if load_deps {
        for dep in &skill.dependencies {
            if let Some(dep_skill) = registry.find(dep) {
                collect_skills(registry, dep_skill, list, seen, true);
            }
        }
    }
}
