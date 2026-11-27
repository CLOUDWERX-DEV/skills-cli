use crate::registry::SkillRegistry;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn execute(
    registry: &SkillRegistry,
    skill_name: &str,
    script_name: &str,
    args: &[String],
) -> Result<()> {
    let skill = match registry.find(skill_name) {
        Some(s) => s,
        None => bail!("Skill '{}' not found. Run 'skills list' to view available skills.", skill_name),
    };

    let scripts = skill.list_scripts();
    if scripts.is_empty() {
        bail!("Skill '{}' has no bundled scripts in 'scripts/'.", skill.name);
    }

    let script_clean = script_name.to_lowercase();
    let script = scripts.iter().find(|s| {
        let name_lower = s.name.to_lowercase();
        name_lower == script_clean
            || name_lower.ends_with(&format!("/{}", script_clean))
            || name_lower.starts_with(&script_clean)
    });

    let script_file = match script {
        Some(s) => &s.path,
        None => {
            eprintln!("Available scripts in skill '{}':", skill.name);
            for s in &scripts {
                eprintln!("  • {}", s.name);
            }
            bail!("Script '{}' not found in skill '{}'.", script_name, skill.name);
        }
    };

    // Ensure executable permission on Unix if possible
    #[cfg(unix)]
    if let Ok(meta) = fs::metadata(script_file) {
        let mut perms = meta.permissions();
        if perms.mode() & 0o111 == 0 {
            perms.set_mode(perms.mode() | 0o755);
            let _ = fs::set_permissions(script_file, perms);
        }
    }

    let mut cmd = build_command(script_file)?;
    cmd.args(args);
    cmd.current_dir(&skill.dir_path);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            let mut hint = String::new();
            if let Ok(content) = fs::read_to_string(script_file) {
                if let Some(first_line) = content.lines().next() {
                    if first_line.starts_with("#!") {
                        hint = format!(" (shebang: '{}')", first_line.trim());
                    }
                }
            }
            bail!(
                "Failed to execute script '{}'{}: {}",
                script_file.display(),
                hint,
                e
            );
        }
    };

    if !status.success() {
        let code = status.code().unwrap_or(1);
        std::process::exit(code);
    }

    Ok(())
}

fn build_command(path: &Path) -> Result<Command> {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "py" => {
                let mut c = Command::new("python3");
                c.arg(path);
                return Ok(c);
            }
            "sh" | "bash" => {
                let mut c = Command::new("bash");
                c.arg(path);
                return Ok(c);
            }
            "fish" => {
                let mut c = Command::new("fish");
                c.arg(path);
                return Ok(c);
            }
            "js" | "mjs" => {
                let mut c = Command::new("node");
                c.arg(path);
                return Ok(c);
            }
            _ => {}
        }
    }

    // Inspect shebang if extension did not match
    if let Ok(content) = fs::read_to_string(path) {
        if let Some(first_line) = content.lines().next() {
            let trimmed = first_line.trim();
            if let Some(rest) = trimmed.strip_prefix("#!") {
                let shebang = rest.trim();
                let parts: Vec<&str> = shebang.split_whitespace().collect();
                if let Some(first) = parts.first() {
                    if *first == "/usr/bin/env" && parts.len() > 1 {
                        let interp = parts[1];
                        let mut c = Command::new(interp);
                        for extra in &parts[2..] {
                            c.arg(extra);
                        }
                        c.arg(path);
                        return Ok(c);
                    } else if !first.is_empty() {
                        let interp = *first;
                        let mut c = Command::new(interp);
                        for extra in &parts[1..] {
                            c.arg(extra);
                        }
                        c.arg(path);
                        return Ok(c);
                    }
                }
            }
        }
        // Plain text file without shebang: fall back to /bin/sh
        let mut c = Command::new("sh");
        c.arg(path);
        return Ok(c);
    }

    Ok(Command::new(path))
}
