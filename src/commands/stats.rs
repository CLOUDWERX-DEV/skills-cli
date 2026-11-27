use crate::output::{bold, cyan, dim, format_bytes, format_tokens, green, magenta, yellow};
use crate::registry::SkillRegistry;
use anyhow::Result;
use serde_json::json;

pub fn execute(registry: &SkillRegistry, as_json: bool) -> Result<()> {
    let stats = registry.global_stats();

    if as_json {
        let val = json!({
            "total_skills": stats.total_skills,
            "search_directories": registry.search_dirs(),
            "total_files": stats.total_files,
            "total_bytes": stats.total_bytes,
            "total_vault_tokens": stats.total_tokens,
            "total_references": stats.total_refs,
            "total_reference_bytes": stats.total_ref_bytes,
            "total_reference_tokens": stats.total_ref_tokens,
            "total_scripts": stats.total_scripts,
            "total_assets": stats.total_assets,
            "total_instruction_bytes": stats.total_md_bytes,
            "total_instruction_tokens": stats.total_md_tokens,
        });
        println!("{}", serde_json::to_string_pretty(&val)?);
        return Ok(());
    }

    println!("{}", bold(&cyan("══════════ Skill Vault Global Statistics ══════════")));
    println!(
        "  Discovered Skills:   {}",
        bold(&green(&stats.total_skills.to_string()))
    );
    println!(
        "  Search Directories:  {}",
        registry.search_dirs().len()
    );
    for d in registry.search_dirs() {
        println!("    {} {}", dim("•"), d.display());
    }

    println!();
    println!("{}", bold("  Content Breakdown:"));
    println!(
        "    • {:<20} {:<10} ({})",
        "Instruction Files:",
        format!("{} files", stats.total_skills),
        format_tokens(stats.total_md_tokens)
    );
    println!(
        "    • {:<20} {:<10} ({} in {})",
        "Reference Docs:",
        format!("{} files", stats.total_refs),
        format_tokens(stats.total_ref_tokens),
        format_bytes(stats.total_ref_bytes)
    );
    println!(
        "    • {:<20} {:<10}",
        "Bundled Scripts:",
        format!("{} scripts", stats.total_scripts)
    );
    println!(
        "    • {:<20} {:<10}",
        "Assets & Templates:",
        format!("{} assets", stats.total_assets)
    );
    println!(
        "    • {:<20} {:<10} ({} in {})",
        bold("Total Vault:"),
        format!("{} files", stats.total_files),
        bold(&magenta(&format_tokens(stats.total_tokens))),
        bold(&format_bytes(stats.total_bytes))
    );

    println!();
    println!("{}", bold("  Context Efficiency Analysis:"));
    let static_tax = stats.total_skills * 120; // ~120 tokens per frontmatter entry
    println!(
        "    • Static System Prompt Tax (if all {} skills preloaded): ~{} tokens",
        stats.total_skills,
        yellow(&static_tax.to_string())
    );
    println!(
        "    • On-Demand Single-Skill CLI Tax:                         {}",
        green("~150 tokens (98.9% context savings!)")
    );
    println!(
        "    • Total Vault Tokens Protected from Accidental Dump:      {}",
        magenta(&format_tokens(stats.total_tokens))
    );

    Ok(())
}
