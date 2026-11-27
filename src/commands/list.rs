use crate::output::{
    bold, cyan, dim, format_tokens, green, magenta, terminal_width, truncate_ellipsis,
    wrap_text, yellow,
};
use crate::registry::SkillRegistry;
use anyhow::Result;
use serde_json::json;

pub fn execute(
    registry: &SkillRegistry,
    filter: Option<&str>,
    short: bool,
    full: bool,
    table: bool,
    as_json: bool,
) -> Result<()> {
    let mut skills: Vec<_> = registry.all().iter().collect();

    if let Some(f) = filter {
        let fl = f.to_lowercase();
        skills.retain(|s| {
            s.name.to_lowercase().contains(&fl)
                || s.description.to_lowercase().contains(&fl)
        });
    }

    if as_json {
        let list: Vec<_> = skills
            .iter()
            .map(|s| {
                let st = s.stats();
                json!({
                    "name": s.name,
                    "description": s.description,
                    "path": s.dir_path,
                    "has_skill_md": s.skill_md_path.is_some(),
                    "instructions_tokens": st.skill_md_tokens,
                    "references_count": st.ref_count,
                    "scripts_count": st.script_count,
                    "assets_count": st.asset_count,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(());
    }

    if short {
        for s in skills {
            println!("{}", s.name);
        }
        return Ok(());
    }

    println!(
        "{} skills discovered across {} search directories\n",
        bold(&skills.len().to_string()),
        registry.search_dirs().len()
    );

    if full {
        for s in skills {
            let st = s.stats();
            println!("{}", bold(&cyan(&s.name)));
            println!("  {} {}", dim("Path:"), s.dir_path.display());
            println!(
                "  {} ~{} tokens | {} refs | {} scripts | {} assets",
                dim("Weight:"),
                st.skill_md_tokens,
                st.ref_count,
                st.script_count,
                st.asset_count
            );
            if !s.description.is_empty() {
                println!("  {} {}", dim("Desc:"), s.description);
            }
            println!();
        }
    } else if table {
        println!(
            "{:<32} {:<10} {:<8} {:<8} {}",
            bold("SKILL"),
            bold("INSTRUCT"),
            bold("REFS"),
            bold("SCRIPTS"),
            bold("DESCRIPTION")
        );
        println!("{}", dim(&"─".repeat(90)));

        for s in skills {
            let st = s.stats();
            let desc_snippet = truncate_ellipsis(&s.description, 45);

            let token_str = format!("~{}t", st.skill_md_tokens);
            let ref_str = if st.ref_count > 0 {
                green(&st.ref_count.to_string())
            } else {
                dim("-")
            };
            let script_str = if st.script_count > 0 {
                yellow(&st.script_count.to_string())
            } else {
                dim("-")
            };

            println!(
                "{:<32} {:<10} {:<8} {:<8} {}",
                cyan(&s.name),
                magenta(&token_str),
                ref_str,
                script_str,
                dim(&desc_snippet)
            );
        }
        println!();
    } else {
        let width = terminal_width().min(120);

        for (i, s) in skills.iter().enumerate() {
            let st = s.stats();
            let tok_str = magenta(&format_tokens(st.skill_md_tokens));
            let mut badges = vec![tok_str];
            if st.ref_count > 0 {
                badges.push(green(&format!("{} refs", st.ref_count)));
            }
            if st.script_count > 0 {
                badges.push(yellow(&format!("{} scripts", st.script_count)));
            }
            if st.asset_count > 0 {
                badges.push(dim(&format!("{} assets", st.asset_count)));
            }

            println!("• {} ({})", bold(&cyan(&s.name)), badges.join(" • "));
            if !s.description.is_empty() {
                let wrapped = wrap_text(&s.description, width, "    ");
                println!("    {}", wrapped);
            } else {
                println!("    {}", dim("(No description in frontmatter)"));
            }

            if i + 1 < skills.len() {
                println!();
            }
        }
        println!();
        println!(
            "Tip: run '{}' to load instructions into context, or '{}' to inspect deep files.",
            cyan("skills load <name...>"),
            cyan("skills ref <name>")
        );
    }

    Ok(())
}
