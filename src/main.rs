mod agents;
mod commands;
mod config;
mod output;
mod registry;
mod skill;

use clap::{Parser, Subcommand};
use config::Config;
use output::init_color;
use registry::SkillRegistry;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "skills",
    version,
    about = "Fast CLI manager and dynamic context loader for agent skills",
    long_about = "Fast CLI manager and dynamic context loader for agent skills.\nProvides instant search, progressive context loading, reference querying, and script execution across local skill repositories."
)]
struct Cli {
    #[arg(
        short = 'd',
        long = "dir",
        help = "Additional directory to search for skills (can be repeated)"
    )]
    dir: Vec<PathBuf>,

    #[arg(short = 'c', long = "config", help = "Custom configuration file path")]
    config: Option<PathBuf>,

    #[arg(long = "no-color", help = "Disable ANSI color output")]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "List available skills across configured directories")]
    List {
        #[arg(help = "Optional filter substring")]
        filter: Option<String>,

        #[arg(short = 's', long = "short", help = "Print only skill names")]
        short: bool,

        #[arg(long = "full", help = "Show full descriptions and paths")]
        full: bool,

        #[arg(long = "table", help = "Display compact single-line table view")]
        table: bool,

        #[arg(long = "json", help = "Output structured JSON")]
        json: bool,
    },

    #[command(about = "Search skills by name, description, or compatibility")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(long = "json", help = "Output structured JSON")]
        json: bool,
    },

    #[command(
        about = "Load skill instructions (SKILL.md) into context (supports multiple skills at once)",
        alias = "get",
        alias = "show",
        alias = "cat",
        alias = "read",
        alias = "view"
    )]
    Load {
        #[arg(required = true, help = "One or more skill names to load")]
        skills: Vec<String>,

        #[arg(long = "raw", help = "Output raw instructions without banners")]
        raw: bool,

        #[arg(long = "with-frontmatter", help = "Preserve YAML frontmatter")]
        with_frontmatter: bool,

        #[arg(short = 'q', long = "quiet", help = "Suppress token estimate banners")]
        quiet: bool,

        #[arg(long = "deps", help = "Automatically load companion dependencies")]
        deps: bool,
    },

    #[command(about = "Inspect deep metadata, file tree, and token footprint for a skill")]
    Info {
        #[arg(required = true, help = "One or more skill names to inspect")]
        skills: Vec<String>,

        #[arg(long = "json", help = "Output structured JSON")]
        json: bool,
    },

    #[command(about = "Inspect, search, or load reference documentation inside a skill")]
    Ref {
        #[arg(help = "Skill name")]
        skill: String,

        #[arg(help = "Reference file path(s) to load, or subcommand (list/get/search/all)")]
        targets: Vec<String>,

        #[arg(long = "all", help = "Load ALL references at once")]
        all: bool,

        #[arg(short = 'f', long = "force", help = "Force loading all references even if large")]
        force: bool,

        #[arg(long = "search", help = "Search text inside reference files")]
        search: Option<String>,

        #[arg(long = "raw", help = "Output raw markdown without banners")]
        raw: bool,

        #[arg(long = "max-tokens", help = "Override safety limit for tokens")]
        max_tokens: Option<usize>,
    },

    #[command(about = "Execute a bundled script from a skill's scripts/ directory")]
    Run {
        #[arg(help = "Skill name")]
        skill: String,

        #[arg(help = "Script name inside scripts/")]
        script: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true, help = "Arguments to pass to the script")]
        args: Vec<String>,
    },

    #[command(about = "List all files inside a skill with category, size, and tokens")]
    Files {
        #[arg(help = "Skill name")]
        skill: String,

        #[arg(short = 'c', long = "category", help = "Filter category (instruction, reference, script, asset)")]
        category: Option<String>,

        #[arg(long = "json", help = "Output structured JSON")]
        json: bool,
    },

    #[command(about = "Show global statistics and token savings across all skills")]
    Stats {
        #[arg(long = "json", help = "Output structured JSON")]
        json: bool,
    },

    #[command(about = "View or modify skills configuration and search paths")]
    Config {
        #[command(subcommand)]
        sub: Option<ConfigSub>,
    },

    #[command(
        about = "Output the built-in meta-skill instructions for operating the skills CLI",
        alias = "prompt",
        alias = "guide",
        alias = "instructions"
    )]
    Skill {
        #[arg(long = "raw", help = "Output raw instructions without banners")]
        raw: bool,

        #[arg(long = "with-frontmatter", help = "Include YAML frontmatter")]
        with_frontmatter: bool,
    },

    #[command(about = "Print the absolute path to a skill or reference file", alias = "path")]
    Which {
        #[arg(help = "Skill name")]
        skill: String,

        #[arg(long = "ref", help = "Relative reference file to locate")]
        ref_file: Option<String>,

        #[arg(long = "script", help = "Bundled script to locate")]
        script: Option<String>,
    },

    #[command(about = "Pin (symlink) skill(s) into agent directories for native slash commands")]
    Pin {
        #[arg(required = true, help = "One or more skills to pin")]
        skills: Vec<String>,

        #[arg(short = 'a', long = "agents", alias = "agent", help = "Target agents (claude,codex,agy,agents,opencode,pi,hermes,all)")]
        agents: Option<String>,
    },

    #[command(about = "Remove pinned skill(s) from agent directories")]
    Unpin {
        #[arg(help = "Specific skill(s) to unpin")]
        skills: Vec<String>,

        #[arg(long = "all", help = "Unpin ALL skills across agents")]
        all: bool,

        #[arg(short = 'a', long = "agents", alias = "agent", help = "Target agents (claude,codex,agy,agents,opencode,pi,hermes,all)")]
        agents: Option<String>,
    },

    #[command(about = "List all currently pinned skills across agents")]
    Pinned {
        #[arg(short = 'a', long = "agents", alias = "agent", help = "Filter agents (claude,codex,agy,agents,opencode,pi,hermes,all)")]
        agents: Option<String>,
    },

    #[command(about = "Run diagnostic audit on vault and agent directories")]
    Doctor {
        #[arg(long = "fix", help = "Automatically repair detected issues")]
        fix: bool,
    },

    #[command(about = "Validate skills against Agent Skills specification")]
    Validate {
        #[arg(help = "Specific skill(s) to validate (defaults to all)")]
        skills: Vec<String>,
    },

    #[command(about = "Create a new skill with a valid SKILL.md to start from")]
    New {
        #[arg(help = "Skill name: lowercase letters, digits and hyphens, like pdf-tools")]
        name: String,

        #[arg(long = "description", help = "One-line description (what it does and when to use it)")]
        description: Option<String>,

        #[arg(long = "to", help = "Folder to create it in (default: the first configured skills folder)")]
        to: Option<PathBuf>,
    },

    #[command(about = "Install skills from a git repository (URL or owner/repo) or a local folder")]
    Install {
        #[arg(help = "A git URL, owner/repo on GitHub, or a local folder")]
        source: String,

        #[arg(long = "path", help = "Look only in this folder inside the source")]
        path: Option<String>,

        #[arg(long = "skill", help = "Install only this skill (can be repeated)")]
        skill: Vec<String>,

        #[arg(long = "to", help = "Folder to install into (default: the first configured skills folder)")]
        to: Option<PathBuf>,

        #[arg(long = "force", help = "Replace skills that are already installed")]
        force: bool,
    },

    #[command(about = "Remove skills from the library (asks for --yes)")]
    Remove {
        #[arg(help = "Skill(s) to remove")]
        skills: Vec<String>,

        #[arg(long = "yes", help = "Really remove them")]
        yes: bool,
    },

    #[command(about = "Generate or install shell completion scripts for fish, bash, or zsh")]
    Completions {
        #[arg(help = "Target shell (fish, bash, zsh)")]
        shell: Option<String>,

        #[arg(long = "install", help = "Install completions into current user's shell config")]
        install: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigSub {
    #[command(about = "Show current configuration and resolved search paths")]
    Show,

    #[command(about = "Print the path to the configuration file")]
    Path,

    #[command(about = "Initialize default configuration file")]
    Init,

    #[command(about = "Add a new search directory to configuration")]
    Add {
        #[arg(help = "Directory path to add")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    init_color(cli.no_color);

    let mut config = Config::load(&cli.dir, cli.config.as_deref())?;

    // Handle completions and config commands before discovering registry if not needed
    if let Commands::Completions { ref shell, install } = cli.command {
        let is_install = install || shell.as_deref() == Some("install");
        let shell_name = if shell.as_deref() == Some("install") {
            None
        } else {
            shell.as_deref()
        };

        if is_install {
            return commands::completions::execute_install(shell_name);
        } else if let Some(sh) = shell_name {
            return commands::completions::execute_print(sh);
        } else {
            return commands::completions::execute_print("fish");
        }
    }

    if let Commands::Config { ref sub } = cli.command {
        let action = match sub {
            Some(ConfigSub::Show) | None => commands::config_cmd::ConfigSubcommand::Show,
            Some(ConfigSub::Path) => commands::config_cmd::ConfigSubcommand::Path,
            Some(ConfigSub::Init) => commands::config_cmd::ConfigSubcommand::Init,
            Some(ConfigSub::Add { path }) => {
                commands::config_cmd::ConfigSubcommand::Add { path: path.clone() }
            }
        };
        return commands::config_cmd::execute(&mut config, action);
    }

    if let Commands::New { ref name, ref description, ref to } = cli.command {
        return commands::manage::new_skill(&config, name, description.as_deref(), to.as_deref());
    }
    if let Commands::Install { ref source, ref path, ref skill, ref to, force } = cli.command {
        return commands::manage::install(&config, source, path.as_deref(), skill, to.as_deref(), force);
    }

    let registry = SkillRegistry::discover(&config);

    match cli.command {
        Commands::List { filter, short, full, table, json } => {
            commands::list::execute(&registry, filter.as_deref(), short, full, table, json)?;
        }
        Commands::Search { query, json } => {
            commands::search::execute(&registry, &query, json)?;
        }
        Commands::Load {
            skills,
            raw,
            with_frontmatter,
            quiet,
            deps,
        } => {
            commands::load::execute(&registry, &skills, raw, with_frontmatter, quiet, deps)?;
        }
        Commands::Info { skills, json } => {
            commands::info::execute(&registry, &skills, json)?;
        }
        Commands::Ref {
            skill,
            targets,
            all,
            force,
            search,
            raw,
            max_tokens,
        } => {
            let (target_skill, action) = resolve_ref_args(
                &skill,
                &targets,
                all,
                force,
                search,
                raw,
                max_tokens,
            )?;
            commands::reference::execute(&registry, &target_skill, action)?;
        }
        Commands::Run { skill, script, args } => {
            commands::run::execute(&registry, &skill, &script, &args)?;
        }
        Commands::Files { skill, category, json } => {
            commands::files::execute(&registry, &skill, category.as_deref(), json)?;
        }
        Commands::Stats { json } => {
            commands::stats::execute(&registry, json)?;
        }
        Commands::Skill { raw, with_frontmatter } => {
            commands::skill_cmd::execute(raw, with_frontmatter)?;
        }
        Commands::Which { skill, ref_file, script } => {
            commands::which::execute(&registry, &skill, ref_file.as_deref(), script.as_deref())?;
        }
        Commands::Pin { skills, agents } => {
            commands::pin::execute_pin(&registry, &skills, agents.as_deref())?;
        }
        Commands::Unpin { skills, all, agents } => {
            commands::pin::execute_unpin(&skills, all, agents.as_deref())?;
        }
        Commands::Pinned { agents } => {
            commands::pin::execute_pinned(agents.as_deref())?;
        }
        Commands::Doctor { fix } => {
            commands::doctor::execute(&config, &registry, fix)?;
        }
        Commands::Validate { skills } => {
            commands::validate::execute(&registry, &skills)?;
        }
        Commands::Remove { skills, yes } => {
            commands::manage::remove(&registry, &skills, yes)?;
        }
        Commands::New { .. } | Commands::Install { .. } => unreachable!(),
        Commands::Completions { .. } => unreachable!(),
        Commands::Config { .. } => unreachable!(),
    }

    Ok(())
}

fn resolve_ref_args(
    skill_arg: &str,
    targets: &[String],
    flag_all: bool,
    flag_force: bool,
    flag_search: Option<String>,
    flag_raw: bool,
    flag_max_tokens: Option<usize>,
) -> anyhow::Result<(String, commands::reference::RefAction)> {
    if let Some(query) = flag_search {
        return Ok((
            skill_arg.to_string(),
            commands::reference::RefAction::Search { query },
        ));
    }

    // Check if skill_arg is actually a verb ("list", "get", "search", "all")
    match skill_arg {
        "list" => {
            if targets.is_empty() {
                anyhow::bail!("Usage: skills ref list <skill>");
            }
            Ok((targets[0].clone(), commands::reference::RefAction::List))
        }
        "all" => {
            if targets.is_empty() {
                anyhow::bail!("Usage: skills ref all <skill>");
            }
            Ok((
                targets[0].clone(),
                commands::reference::RefAction::All {
                    force: flag_force,
                    max_tokens: flag_max_tokens,
                    raw: flag_raw,
                },
            ))
        }
        "search" => {
            if targets.is_empty() {
                anyhow::bail!("Usage: skills ref search <skill> <query>");
            }
            let skill_name = targets[0].clone();
            let query = if targets.len() > 1 {
                targets[1..].join(" ")
            } else {
                String::new()
            };
            Ok((
                skill_name,
                commands::reference::RefAction::Search { query },
            ))
        }
        "get" => {
            if targets.is_empty() {
                anyhow::bail!("Usage: skills ref get <skill> <file...>");
            }
            let skill_name = targets[0].clone();
            let paths = if targets.len() > 1 {
                targets[1..].to_vec()
            } else {
                Vec::new()
            };
            Ok((
                skill_name,
                commands::reference::RefAction::Get {
                    paths,
                    raw: flag_raw,
                },
            ))
        }
        _ => {
            // skill_arg is the actual skill name (e.g. "pdf-tools")
            let skill_name = skill_arg.to_string();

            if flag_all {
                return Ok((
                    skill_name,
                    commands::reference::RefAction::All {
                        force: flag_force,
                        max_tokens: flag_max_tokens,
                        raw: flag_raw,
                    },
                ));
            }

            if targets.is_empty() {
                return Ok((skill_name, commands::reference::RefAction::List));
            }

            let first = &targets[0];
            match first.as_str() {
                "list" => Ok((skill_name, commands::reference::RefAction::List)),
                "all" => Ok((
                    skill_name,
                    commands::reference::RefAction::All {
                        force: flag_force,
                        max_tokens: flag_max_tokens,
                        raw: flag_raw,
                    },
                )),
                "search" => {
                    let query = if targets.len() > 1 {
                        targets[1..].join(" ")
                    } else {
                        String::new()
                    };
                    Ok((
                        skill_name,
                        commands::reference::RefAction::Search { query },
                    ))
                }
                "get" => {
                    let paths = if targets.len() > 1 {
                        targets[1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    Ok((
                        skill_name,
                        commands::reference::RefAction::Get {
                            paths,
                            raw: flag_raw,
                        },
                    ))
                }
                _ => {
                    // All targets are treated as reference paths to load!
                    Ok((
                        skill_name,
                        commands::reference::RefAction::Get {
                            paths: targets.to_vec(),
                            raw: flag_raw,
                        },
                    ))
                }
            }
        }
    }
}
