use crate::registry::SkillRegistry;
use anyhow::{bail, Result};

pub fn execute(
    registry: &SkillRegistry,
    skill_name: &str,
    ref_file: Option<&str>,
    script_file: Option<&str>,
) -> Result<()> {
    let skill = match registry.find(skill_name) {
        Some(s) => s,
        None => bail!("Skill '{}' not found in any search path.", skill_name),
    };

    if let Some(rf) = ref_file {
        let refs = skill.list_references();
        let target = rf.to_lowercase();
        if let Some(r) = refs.iter().find(|r| {
            r.rel_path.to_lowercase() == target
                || r.rel_path.to_lowercase().ends_with(&target)
                || r.rel_path.to_lowercase().contains(&target)
        }) {
            println!("{}", r.path.display());
            return Ok(());
        }
        bail!(
            "Reference file '{}' not found in skill '{}'. Run 'skills ref list {}' to see available references.",
            rf,
            skill.name,
            skill.name
        );
    }

    if let Some(sf) = script_file {
        let scripts = skill.list_scripts();
        let target = sf.to_lowercase();
        if let Some(s) = scripts.iter().find(|s| {
            s.name.to_lowercase() == target || s.name.to_lowercase().starts_with(&target)
        }) {
            println!("{}", s.path.display());
            return Ok(());
        }
        bail!(
            "Script '{}' not found in skill '{}'. Run 'skills info {}' to see available scripts.",
            sf,
            skill.name,
            skill.name
        );
    }

    println!("{}", skill.dir_path.display());
    Ok(())
}
