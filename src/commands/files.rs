use crate::output::{bold, cyan, dim, format_bytes, format_tokens, magenta, yellow};
use crate::registry::SkillRegistry;
use anyhow::{bail, Result};

pub fn execute(
    registry: &SkillRegistry,
    skill_name: &str,
    category: Option<&str>,
    as_json: bool,
) -> Result<()> {
    let skill = match registry.find(skill_name) {
        Some(s) => s,
        None => bail!("Skill '{}' not found.", skill_name),
    };

    let mut entries = skill.list_all_files();
    if let Some(cat) = category {
        let cat_lower = cat.to_lowercase();
        entries.retain(|e| e.category.to_lowercase() == cat_lower);
    }

    if as_json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    println!(
        "Files for skill '{}' ({} files):\n",
        bold(&cyan(&skill.name)),
        entries.len()
    );

    println!(
        "{:<45} {:<12} {:<12} {:<10} {}",
        bold("PATH"),
        bold("CATEGORY"),
        bold("TOKENS"),
        bold("SIZE"),
        bold("LINES")
    );
    println!("{}", dim(&"─".repeat(88)));

    let mut total_bytes = 0;
    let mut total_tokens = 0;
    let mut total_lines = 0;

    for e in &entries {
        total_bytes += e.bytes;
        total_tokens += e.tokens;
        total_lines += e.lines;

        let cat_colored = match e.category.as_str() {
            "instruction" => yellow(&e.category),
            "reference" => cyan(&e.category),
            "script" => yellow(&e.category),
            "asset" => dim(&e.category),
            _ => dim(&e.category),
        };

        println!(
            "{:<45} {:<12} {:<12} {:<10} {}",
            e.rel_path,
            cat_colored,
            magenta(&format_tokens(e.tokens)),
            dim(&format_bytes(e.bytes)),
            dim(&format!("{} L", e.lines))
        );
    }

    println!("{}", dim(&"─".repeat(88)));
    println!(
        "{:<45} {:<12} {:<12} {:<10} {}",
        bold("TOTAL"),
        format!("{} files", entries.len()),
        bold(&magenta(&format_tokens(total_tokens))),
        bold(&format_bytes(total_bytes)),
        bold(&format!("{} L", total_lines))
    );

    Ok(())
}
