use crate::output::{bold, cyan, dim, green, red, yellow};
use crate::registry::SkillRegistry;
use crate::skill::Skill;
use anyhow::{bail, Result};
use regex::Regex;
use std::fs;

pub fn execute(registry: &SkillRegistry, targets: &[String]) -> Result<()> {
    let skills_to_validate: Vec<&Skill> = if targets.is_empty() {
        registry.all().iter().collect()
    } else {
        let mut matched = Vec::new();
        for t in targets {
            if let Some(s) = registry.find(t) {
                matched.push(s);
            } else {
                eprintln!("{} Skill '{}' not found in vault.", yellow("Warning:"), t);
            }
        }
        if matched.is_empty() {
            bail!("None of the specified skills were found.");
        }
        matched
    };

    println!(
        "Validating {} skill(s) against Agent Skills standard:\n",
        bold(&skills_to_validate.len().to_string())
    );

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let name_regex = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

    for skill in &skills_to_validate {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. SKILL.md existence
        let md_path = match skill.skill_md_path {
            Some(ref p) => p,
            None => {
                errors.push("Missing SKILL.md or skill.md file in directory.".to_string());
                print_report(skill.name.as_str(), &errors, &warnings);
                total_errors += errors.len();
                continue;
            }
        };

        // 2. Name validation
        if !name_regex.is_match(&skill.name) {
            warnings.push(format!(
                "Skill name '{}' does not match Agent Skills naming convention (lowercase a-z, 0-9, hyphens only).",
                skill.name
            ));
        }

        let dir_folder_name = skill
            .dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if !dir_folder_name.eq_ignore_ascii_case(&skill.name) {
            warnings.push(format!(
                "Frontmatter name '{}' differs from directory name '{}'.",
                skill.name, dir_folder_name
            ));
        }

        // 3. Description validation
        if skill.description.trim().is_empty() {
            errors.push("Description is empty in YAML frontmatter.".to_string());
        } else if skill.description.len() > 1024 {
            warnings.push(format!(
                "Description exceeds recommended 1,024 char limit (currently {} chars).",
                skill.description.len()
            ));
        }

        // 4. Dependency check
        for dep in &skill.dependencies {
            if registry.get(dep).is_none() {
                warnings.push(format!(
                    "Declared dependency '{}' was not found in the skills vault.",
                    dep
                ));
            }
        }

        // 5. Relative markdown link checker
        if let Ok(md_content) = fs::read_to_string(md_path) {
            for cap in link_regex.captures_iter(&md_content) {
                let link_target = &cap[2];
                // Check local relative file links (ignore http/https/mailto/#anchors)
                if !link_target.starts_with("http://")
                    && !link_target.starts_with("https://")
                    && !link_target.starts_with("mailto:")
                    && !link_target.starts_with('#')
                {
                    let clean_target = link_target.split('#').next().unwrap_or(link_target).trim();
                    if clean_target.is_empty() {
                        continue;
                    }
                    let resolved = skill.dir_path.join(clean_target);
                    if !resolved.exists() {
                        warnings.push(format!(
                            "Broken link in SKILL.md: '{}' (file not found at {})",
                            link_target,
                            resolved.display()
                        ));
                    }
                }
            }
        }

        // 6. Scripts check
        for script in skill.list_scripts() {
            if let Ok(content) = fs::read_to_string(&script.path) {
                let first_line = content.lines().next().unwrap_or("");
                let ext = script.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if (ext == "sh" || ext == "bash" || ext == "py") && !first_line.starts_with("#!") {
                    warnings.push(format!(
                        "Script 'scripts/{}' is missing a shebang line (e.g. #!/usr/bin/env {}).",
                        script.name,
                        if ext == "py" { "python3" } else { "bash" }
                    ));
                }
            }
        }

        total_errors += errors.len();
        total_warnings += warnings.len();
        print_report(&skill.name, &errors, &warnings);
    }

    println!("\n{}", dim(&"─".repeat(70)));
    println!(
        "Validated {} skills: {} error(s), {} warning(s).",
        skills_to_validate.len(),
        if total_errors > 0 { red(&total_errors.to_string()) } else { green("0") },
        if total_warnings > 0 { yellow(&total_warnings.to_string()) } else { green("0") }
    );

    if total_errors > 0 {
        bail!("Validation completed with {} error(s).", total_errors);
    }

    Ok(())
}

fn print_report(name: &str, errors: &[String], warnings: &[String]) {
    if errors.is_empty() && warnings.is_empty() {
        println!("  {} {:<32} {}", green("✓"), cyan(name), dim("Valid"));
    } else if !errors.is_empty() {
        println!("  {} {:<32} {}", red("✗"), bold(name), red(&format!("({} error(s))", errors.len())));
        for e in errors {
            println!("     {} {}", red("•"), e);
        }
        for w in warnings {
            println!("     {} {}", yellow("•"), w);
        }
    } else {
        println!("  {} {:<32} {}", yellow("!"), cyan(name), yellow(&format!("({} warning(s))", warnings.len())));
        for w in warnings {
            println!("     {} {}", yellow("•"), w);
        }
    }
}
