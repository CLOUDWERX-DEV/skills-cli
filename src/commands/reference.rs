use crate::output::{bold, cyan, dim, format_bytes, format_tokens, green, magenta, yellow};
use crate::registry::SkillRegistry;
use anyhow::{bail, Result};
use std::fs;

#[derive(Debug, Clone)]
pub enum RefAction {
    List,
    Get { paths: Vec<String>, raw: bool },
    Search { query: String },
    All { force: bool, max_tokens: Option<usize>, raw: bool },
}

pub fn execute(registry: &SkillRegistry, skill_name: &str, action: RefAction) -> Result<()> {
    let skill = match registry.find(skill_name) {
        Some(s) => s,
        None => bail!("Skill '{}' not found. Run 'skills list' to view available skills.", skill_name),
    };

    let refs = skill.list_references();
    if refs.is_empty() {
        println!("Skill '{}' does not have a 'references/' directory or contains no reference files.", skill.name);
        return Ok(());
    }

    match action {
        RefAction::List => {
            println!(
                "References for skill '{}' ({} files, {} total):\n",
                bold(&cyan(&skill.name)),
                refs.len(),
                format_tokens(refs.iter().map(|r| r.tokens).sum())
            );

            println!(
                "{:<50} {:<12} {:<10} {}",
                bold("REFERENCE FILE"),
                bold("TOKENS"),
                bold("SIZE"),
                bold("LINES")
            );
            println!("{}", dim(&"─".repeat(82)));

            for r in &refs {
                println!(
                    "{:<50} {:<12} {:<10} {}",
                    cyan(&r.rel_path),
                    magenta(&format_tokens(r.tokens)),
                    dim(&format_bytes(r.bytes)),
                    dim(&format!("{} lines", r.lines))
                );
            }

            println!();
            println!(
                "Load specific file:  {}",
                green(&format!("skills ref get {} <file>", skill.name))
            );
            println!(
                "Load multiple files: {}",
                green(&format!("skills ref get {} <file1> <file2...>", skill.name))
            );
            println!(
                "Search references:   {}",
                cyan(&format!("skills ref search {} <term>", skill.name))
            );
            println!(
                "Load ALL references: {}",
                yellow(&format!("skills ref all {} [--force]", skill.name))
            );
        }

        RefAction::Get { paths, raw } => {
            if paths.is_empty() {
                bail!("No reference file path specified. Run 'skills ref list {}' to see options.", skill.name);
            }

            let mut matched = Vec::new();
            let mut not_found = Vec::new();

            for requested in &paths {
                let req_lower = requested.to_lowercase();
                // 1. Exact match relative path
                if let Some(r) = refs.iter().find(|r| r.rel_path.to_lowercase() == req_lower) {
                    matched.push(r);
                    continue;
                }
                // 2. Ends with match (e.g. "guide.md" matches "docs/api/guide.md")
                if let Some(r) = refs.iter().find(|r| r.rel_path.to_lowercase().ends_with(&req_lower)) {
                    matched.push(r);
                    continue;
                }
                // 3. Substring / fuzzy match
                let matches: Vec<_> = refs.iter().filter(|r| r.rel_path.to_lowercase().contains(&req_lower)).collect();
                if let Some(first) = matches.first() {
                    matched.push(*first); // the first of several is taken, and sorted order keeps that predictable
                } else {
                    not_found.push(requested.clone());
                }
            }

            if !not_found.is_empty() {
                eprintln!(
                    "{} Could not find reference file(s): {}",
                    yellow("Warning:"),
                    not_found.join(", ")
                );
            }

            if matched.is_empty() {
                bail!("None of the requested reference files were found in skill '{}'.", skill.name);
            }

            let total_tok: usize = matched.iter().map(|m| m.tokens).sum();

            if !raw && matched.len() > 1 {
                println!("<!-- ===================================================================== -->");
                println!(
                    "<!-- LOADED {} REFERENCES FOR SKILL: {} | Total tokens: {:<20} -->",
                    matched.len(),
                    skill.name,
                    format_tokens(total_tok)
                );
                for m in &matched {
                    println!("<!--   • {:<40} ({}) -->", m.rel_path, format_tokens(m.tokens));
                }
                println!("<!-- ===================================================================== -->\n");
            }

            for (i, r) in matched.iter().enumerate() {
                let content = match fs::read_to_string(&r.path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed reading {}: {e}", r.path.display());
                        continue;
                    }
                };

                if !raw {
                    println!("<!-- ===================================================================== -->");
                    println!(
                        "<!-- REFERENCE: {} / {} | Tokens: {} -->",
                        skill.name,
                        r.rel_path,
                        format_tokens(r.tokens)
                    );
                    println!("<!-- Path: {} -->", r.path.display());
                    println!("<!-- ===================================================================== -->\n");
                }

                println!("{}", content.trim());

                if i + 1 < matched.len() {
                    println!("\n\n");
                }
            }
        }

        RefAction::Search { query } => {
            let q_lower = query.to_lowercase();
            let mut matches_found = 0;

            println!(
                "Searching references of '{}' for '{}':\n",
                bold(&cyan(&skill.name)),
                yellow(&query)
            );

            for r in &refs {
                let content = match fs::read_to_string(&r.path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let mut hits = Vec::new();
                for (line_num, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&q_lower) {
                        hits.push((line_num + 1, line.trim()));
                    }
                }

                if !hits.is_empty() {
                    matches_found += hits.len();
                    println!("{} ({})", bold(&cyan(&r.rel_path)), dim(&format!("{} hits", hits.len())));
                    for (lno, text) in hits.iter().take(5) {
                        println!("  {}  {}", dim(&format!("L{lno}:")), text);
                    }
                    if hits.len() > 5 {
                        println!("  {}", dim(&format!("... and {} more occurrences", hits.len() - 5)));
                    }
                    println!();
                }
            }

            if matches_found == 0 {
                println!("No matches found for '{}' in references.", query);
            } else {
                println!(
                    "Found {} total occurrences. Load specific reference with '{}'.",
                    matches_found,
                    green(&format!("skills ref get {} <rel_path>", skill.name))
                );
            }
        }

        RefAction::All { force, max_tokens, raw } => {
            let total_tok: usize = refs.iter().map(|r| r.tokens).sum();
            let total_bytes: u64 = refs.iter().map(|r| r.bytes).sum();
            let limit = max_tokens.unwrap_or(30_000);

            if total_tok > limit && !force {
                eprintln!(
                    "{} Skill '{}' references are very large: {} files, {}, {} tokens!",
                    yellow("Safety Guardrail:"),
                    skill.name,
                    refs.len(),
                    format_bytes(total_bytes),
                    bold(&format_tokens(total_tok))
                );
                eprintln!("Dumping all references at once may overflow or severely bloat the agent's context.");
                eprintln!();
                eprintln!("To load anyway, run:");
                eprintln!("  {}", green(&format!("skills ref all {} --force", skill.name)));
                eprintln!();
                eprintln!("Or search / load targeted references:");
                eprintln!("  {}", cyan(&format!("skills ref search {} <topic>", skill.name)));
                eprintln!("  {}", cyan(&format!("skills ref get {} <file1> <file2...>", skill.name)));
                bail!("Operation aborted to protect context window. Use --force to override.");
            }

            if !raw {
                println!("<!-- ===================================================================== -->");
                println!(
                    "<!-- BATCH DUMP: ALL {} REFERENCES FOR SKILL '{}' -->",
                    refs.len(),
                    skill.name
                );
                println!(
                    "<!-- Total Size: {} | Total Estimated Tokens: {} -->",
                    format_bytes(total_bytes),
                    format_tokens(total_tok)
                );
                println!("<!-- ===================================================================== -->\n");
            }

            for (i, r) in refs.iter().enumerate() {
                let content = match fs::read_to_string(&r.path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed reading {}: {e}", r.path.display());
                        continue;
                    }
                };

                if !raw {
                    println!("<!-- ===================================================================== -->");
                    println!(
                        "<!-- [{}/{}] REF: {} | {} | {} -->",
                        i + 1,
                        refs.len(),
                        r.rel_path,
                        format_tokens(r.tokens),
                        format_bytes(r.bytes)
                    );
                    println!("<!-- ===================================================================== -->\n");
                }

                println!("{}", content.trim());
                println!("\n\n");
            }
        }
    }

    Ok(())
}
