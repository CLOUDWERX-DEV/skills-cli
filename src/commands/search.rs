use crate::output::{bold, cyan, dim, green, yellow};
use crate::registry::SkillRegistry;
use anyhow::Result;
use serde_json::json;

pub fn execute(registry: &SkillRegistry, query: &str, as_json: bool) -> Result<()> {
    let matches = registry.search(query);

    if as_json {
        let list: Vec<_> = matches
            .iter()
            .map(|(s, score)| {
                let st = s.stats();
                json!({
                    "name": s.name,
                    "score": score,
                    "description": s.description,
                    "path": s.dir_path,
                    "instruction_tokens": st.skill_md_tokens,
                    "references_count": st.ref_count,
                    "scripts_count": st.script_count,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(());
    }

    if matches.is_empty() {
        println!("No skills matched '{}'.", yellow(query));
        println!("Run '{}' to view all available skills.", cyan("skills list"));
        return Ok(());
    }

    println!(
        "Found {} matching skill(s) for '{}':\n",
        bold(&matches.len().to_string()),
        cyan(query)
    );

    for (s, score) in matches {
        let st = s.stats();
        println!(
            "{} {}  {}",
            bold(&cyan(&s.name)),
            dim(&format!("(score: {score})")),
            dim(&format!("~{} tok | {} refs | {} scripts", st.skill_md_tokens, st.ref_count, st.script_count))
        );
        if !s.description.is_empty() {
            println!("   {}", s.description);
        }
        println!("   {} {}", dim("Load with:"), green(&format!("skills load {}", s.name)));
        println!();
    }

    Ok(())
}
