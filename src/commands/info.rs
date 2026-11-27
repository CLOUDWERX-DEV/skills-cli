use crate::output::{bold, cyan, dim, format_bytes, format_tokens, green, magenta, yellow};
use crate::registry::SkillRegistry;
use anyhow::{bail, Result};
use serde_json::json;

pub fn execute(registry: &SkillRegistry, names: &[String], as_json: bool) -> Result<()> {
    if names.is_empty() {
        bail!("No skill name provided. Usage: skills info <skill1> [skill2 ...]");
    }

    let mut found = Vec::new();
    for name in names {
        if let Some(skill) = registry.find(name) {
            found.push(skill);
        } else {
            eprintln!("{} Skill '{}' not found.", yellow("Warning:"), name);
        }
    }

    if found.is_empty() {
        bail!("None of the specified skills were found.");
    }

    if as_json {
        let list: Vec<_> = found
            .iter()
            .map(|s| {
                let st = s.stats();
                json!({
                    "name": s.name,
                    "description": s.description,
                    "path": s.dir_path,
                    "skill_md_path": s.skill_md_path,
                    "license": s.license,
                    "compatibility": s.compatibility,
                    "allowed_tools": s.allowed_tools,
                    "metadata": s.metadata,
                    "stats": st,
                    "scripts": s.list_scripts().iter().map(|sc| &sc.name).collect::<Vec<_>>(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(());
    }

    for (i, skill) in found.iter().enumerate() {
        let st = skill.stats();
        let refs = skill.list_references();
        let scripts = skill.list_scripts();
        let _assets = skill.list_assets();

        println!("{}", bold(&cyan(&format!("═══ Skill: {} ═══", skill.name))));
        println!("  {}  {}", bold("Path:"), skill.dir_path.display());
        if let Some(ref md) = skill.skill_md_path {
            println!("  {}  {}", bold("File:"), md.display());
        }
        if !skill.description.is_empty() {
            println!("  {}  {}", bold("Desc:"), skill.description);
        }
        if let Some(ref lic) = skill.license {
            println!("  {}  {}", bold("License:"), lic);
        }
        if let Some(ref comp) = skill.compatibility {
            println!("  {}  {}", bold("Compat:"), comp);
        }
        if !skill.allowed_tools.is_empty() {
            println!("  {}  {}", bold("Tools:"), skill.allowed_tools.join(", "));
        }
        if !skill.dependencies.is_empty() {
            println!("  {}  {}", bold("Depends:"), cyan(&skill.dependencies.join(", ")));
        }
        if !skill.metadata.is_empty() {
            println!("  {}", bold("Metadata:"));
            for (k, v) in &skill.metadata {
                println!("    {} = {}", dim(k), v);
            }
        }

        println!();
        println!("  {}", bold("Structure & Token Footprint:"));
        println!(
            "    • {:<14} {:<12} ({:<10} {:>8})",
            "SKILL.md:",
            format_tokens(st.skill_md_tokens),
            format_bytes(st.skill_md_bytes),
            format!("{} lines", st.skill_md_lines)
        );
        println!(
            "    • {:<14} {:<12} ({:<10} {:>8})",
            "References:",
            format_tokens(st.ref_tokens),
            format_bytes(st.ref_bytes),
            format!("{} files", st.ref_count)
        );
        println!(
            "    • {:<14} {:<12} ({:<10} {:>8})",
            "Scripts:",
            "-",
            "-",
            format!("{} files", st.script_count)
        );
        println!(
            "    • {:<14} {:<12} ({:<10} {:>8})",
            "Assets:",
            "-",
            "-",
            format!("{} files", st.asset_count)
        );
        println!(
            "    • {:<14} {:<12} ({:<10} {:>8})",
            bold("Total Vault:"),
            magenta(&format_tokens(st.total_tokens)),
            format_bytes(st.total_bytes),
            format!("{} files", st.total_files)
        );

        if !scripts.is_empty() {
            println!();
            println!("  {}", bold("Bundled Scripts:"));
            for sc in &scripts {
                println!(
                    "    {} {} (run with 'skills run {} {}')",
                    yellow("•"),
                    sc.name,
                    skill.name,
                    sc.name
                );
            }
        }

        if !refs.is_empty() {
            println!();
            println!("  {} (sample):", bold("Reference Documents"));
            for r in refs.iter().take(8) {
                println!(
                    "    {} {:<40} {:<10} {:<10}",
                    dim("•"),
                    r.rel_path,
                    format_tokens(r.tokens),
                    dim(&format!("{} lines", r.lines))
                );
            }
            if refs.len() > 8 {
                println!(
                    "    {} ... and {} more reference files (view with 'skills ref list {}')",
                    dim("•"),
                    refs.len() - 8,
                    skill.name
                );
            }
        }

        println!();
        println!("  {}", bold("Quick Actions:"));
        println!("    Load instructions: {}", green(&format!("skills load {}", skill.name)));
        println!("    List references:   {}", cyan(&format!("skills ref list {}", skill.name)));
        if !refs.is_empty() {
            println!("    Search references: {}", cyan(&format!("skills ref search {} <query>", skill.name)));
        }
        if !scripts.is_empty() {
            println!("    Run script:        {}", yellow(&format!("skills run {} <script>", skill.name)));
        }

        if i + 1 < found.len() {
            println!("\n{}\n", dim(&"─".repeat(70)));
        }
    }

    Ok(())
}
