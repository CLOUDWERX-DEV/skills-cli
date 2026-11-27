use crate::config::Config;
use crate::output::{bold, cyan, dim, green, yellow};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ConfigSubcommand {
    Show,
    Path,
    Init,
    Add { path: PathBuf },
}

pub fn execute(config: &mut Config, cmd: ConfigSubcommand) -> Result<()> {
    match cmd {
        ConfigSubcommand::Show => {
            println!("{}", bold(&cyan("Skills Configuration:")));
            println!("  {} {}", dim("Config File:"), config.config_file_path.display());
            println!(
                "  {} {}",
                dim("File Exists:"),
                if config.config_file_path.exists() {
                    green("yes")
                } else {
                    yellow("no (using default resolution)")
                }
            );
            println!("  {} {} tokens", dim("Max Ref Tokens:"), config.max_ref_tokens);
            println!();
            println!("{}", bold("Active Search Paths (in priority order):"));
            for (i, p) in config.skills_dirs.iter().enumerate() {
                let status = if p.is_dir() {
                    green("active")
                } else {
                    dim("not found")
                };
                println!("  {}. {:<55} [{}]", i + 1, p.display(), status);
            }
        }
        ConfigSubcommand::Path => {
            println!("{}", config.config_file_path.display());
        }
        ConfigSubcommand::Init => {
            config.save_default()?;
            println!(
                "{} Initialized configuration at {}",
                green("Success:"),
                config.config_file_path.display()
            );
        }
        ConfigSubcommand::Add { path } => {
            config.add_dir(&path)?;
            println!(
                "{} Added '{}' to {}",
                green("Success:"),
                path.display(),
                config.config_file_path.display()
            );
        }
    }
    Ok(())
}
