use crate::output::{estimate_tokens, format_tokens};
use anyhow::Result;

pub const BUILTIN_SKILL_MD: &str = include_str!("../../skills/skills-cli/SKILL.md");

pub fn execute(raw: bool, with_frontmatter: bool) -> Result<()> {
    let (_fm, body) = parse_frontmatter(BUILTIN_SKILL_MD);

    let content = if with_frontmatter {
        BUILTIN_SKILL_MD.trim()
    } else {
        body.trim()
    };

    let tok = estimate_tokens(content);

    if !raw {
        println!("<!-- ===================================================================== -->");
        println!(
            "<!-- BUILT-IN META-SKILL: skills-cli | Tokens: {:<29} -->",
            format_tokens(tok)
        );
        println!("<!-- Source: embedded in the `skills` binary                                -->");
        println!("<!-- Run 'skills skill' anytime to recall CLI capabilities and workflows -->");
        println!("<!-- ===================================================================== -->\n");
    }

    println!("{}", content);

    Ok(())
}

fn parse_frontmatter(content: &str) -> (String, String) {
    let trimmed = content.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end_idx) = rest.find("\n---") {
            let fm = &rest[..end_idx];
            let after = &rest[end_idx + 4..];
            let body = after.trim_start_matches('\r').trim_start_matches('\n');
            return (fm.trim().to_string(), body.to_string());
        }
    }
    (String::new(), content.to_string())
}
