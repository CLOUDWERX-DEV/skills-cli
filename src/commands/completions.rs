use crate::output::{bold, cyan, green};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Fish,
    Bash,
    Zsh,
}

impl ShellKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "fish" => Some(ShellKind::Fish),
            "bash" => Some(ShellKind::Bash),
            "zsh" => Some(ShellKind::Zsh),
            _ => None,
        }
    }

    pub fn detect() -> Option<Self> {
        if let Ok(shell_var) = std::env::var("SHELL") {
            if shell_var.ends_with("/fish") {
                return Some(ShellKind::Fish);
            } else if shell_var.ends_with("/bash") {
                return Some(ShellKind::Bash);
            } else if shell_var.ends_with("/zsh") {
                return Some(ShellKind::Zsh);
            }
        }
        None
    }

    pub fn name(&self) -> &'static str {
        match self {
            ShellKind::Fish => "fish",
            ShellKind::Bash => "bash",
            ShellKind::Zsh => "zsh",
        }
    }

    pub fn default_install_path(&self) -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        match self {
            ShellKind::Fish => Some(home.join(".config").join("fish").join("completions").join("skills.fish")),
            ShellKind::Bash => Some(
                home.join(".local")
                    .join("share")
                    .join("bash-completion")
                    .join("completions")
                    .join("skills"),
            ),
            ShellKind::Zsh => Some(home.join(".zsh").join("completions").join("_skills")),
        }
    }
}

pub fn generate(shell: ShellKind) -> String {
    match shell {
        ShellKind::Fish => FISH_COMPLETIONS.to_string(),
        ShellKind::Bash => BASH_COMPLETIONS.to_string(),
        ShellKind::Zsh => ZSH_COMPLETIONS.to_string(),
    }
}

pub fn execute_print(shell_name: &str) -> Result<()> {
    let shell = match ShellKind::parse(shell_name) {
        Some(s) => s,
        None => bail!("Unsupported shell '{}'. Supported: fish, bash, zsh", shell_name),
    };
    print!("{}", generate(shell));
    Ok(())
}

pub fn execute_install(shell_opt: Option<&str>) -> Result<()> {
    let shell = if let Some(s) = shell_opt {
        ShellKind::parse(s).context("Invalid shell specified. Supported: fish, bash, zsh")?
    } else {
        ShellKind::detect().unwrap_or(ShellKind::Fish)
    };

    let target_path = shell
        .default_install_path()
        .context("Could not determine user HOME directory")?;

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Creating directory {}", parent.display()))?;
    }

    let script = generate(shell);
    fs::write(&target_path, script)
        .with_context(|| format!("Writing completion script to {}", target_path.display()))?;

    println!(
        "{} Installed {} completions to:\n  {}",
        green("✓ Success:"),
        bold(shell.name()),
        cyan(&target_path.display().to_string())
    );

    match shell {
        ShellKind::Fish => {
            println!("\nCompletions take effect immediately in new fish shells.");
            println!("To reload in your current session: {}", cyan("source ~/.config/fish/completions/skills.fish"));
        }
        ShellKind::Bash => {
            println!("\nTo enable in current shell: {}", cyan(&format!("source {}", target_path.display())));
        }
        ShellKind::Zsh => {
            println!("\nEnsure your ~/.zshrc contains:");
            println!("  {}", cyan("fpath=(~/.zsh/completions $fpath)"));
            println!("  {}", cyan("autoload -U compinit && compinit"));
        }
    }

    Ok(())
}

const FISH_COMPLETIONS: &str = r#"# fish completion for skills CLI
function __fish_skills_skills
    skills list -s 2>/dev/null
end

function __fish_skills_subcommands
    echo list\t"List all available skills"
    echo search\t"Search skills by keyword"
    echo load\t"Load skill instructions into context"
    echo get\t"Load skill instructions into context"
    echo show\t"Show skill instructions"
    echo cat\t"Cat skill instructions"
    echo read\t"Read skill instructions"
    echo view\t"View skill instructions"
    echo info\t"Inspect deep skill metadata and token weights"
    echo ref\t"Query and load reference documents"
    echo run\t"Execute bundled scripts"
    echo files\t"List all files in a skill"
    echo stats\t"Global vault statistics"
    echo config\t"Manage search directories"
    echo skill\t"Print embedded meta-skill instructions"
    echo which\t"Print absolute path to a skill or reference"
    echo path\t"Print absolute path to a skill or reference"
    echo pin\t"Symlink skill into agent directories"
    echo unpin\t"Remove skill from agent directories"
    echo pinned\t"Show currently pinned skills"
    echo doctor\t"Audit vault and agent directory health"
    echo validate\t"Validate skills against Agent Skills spec"
    echo completions\t"Generate or install shell completions"
end

complete -c skills -f -n "__fish_use_subcommand" -a "(__fish_skills_subcommands)"

# Skill argument completions
complete -c skills -f -n "__fish_seen_subcommand_from load get show cat read view info files which path pin unpin validate" -a "(__fish_skills_skills)"
complete -c skills -f -n "__fish_seen_subcommand_from ref" -a "(__fish_skills_skills)"
complete -c skills -f -n "__fish_seen_subcommand_from run" -a "(__fish_skills_skills)"

# Subcommand-specific flags
complete -c skills -n "__fish_seen_subcommand_from list" -s s -l short -d "Output skill names only"
complete -c skills -n "__fish_seen_subcommand_from list" -l full -d "Show full descriptions and paths"
complete -c skills -n "__fish_seen_subcommand_from list" -l table -d "Display compact single-line table view"
complete -c skills -n "__fish_seen_subcommand_from list" -l json -d "Output structured JSON"
complete -c skills -n "__fish_seen_subcommand_from load get show cat read view" -l raw -d "Output raw markdown"
complete -c skills -n "__fish_seen_subcommand_from load get show cat read view" -l with-frontmatter -d "Include YAML frontmatter"
complete -c skills -n "__fish_seen_subcommand_from load get show cat read view" -l deps -d "Load companion dependencies recursively"
complete -c skills -n "__fish_seen_subcommand_from load get show cat read view" -s q -l quiet -d "Suppress token banners"
complete -c skills -n "__fish_seen_subcommand_from pin unpin pinned" -s a -l agents -d "Target agents (claude,codex,agy,agents,opencode,pi,hermes,all)"
complete -c skills -n "__fish_seen_subcommand_from unpin" -l all -d "Unpin all skills"
complete -c skills -n "__fish_seen_subcommand_from doctor" -l fix -d "Automatically fix detected issues"
complete -c skills -n "__fish_seen_subcommand_from completions" -a "fish bash zsh install"
"#;

const BASH_COMPLETIONS: &str = r#"# bash completion for skills CLI
_skills_completions() {
    local cur prev words cword
    _init_completion || return

    local subcommands="list search load get show cat read view info ref run files stats config skill which path pin unpin pinned doctor validate completions"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$subcommands" -- "$cur") )
        return 0
    fi

    local sub="${words[1]}"
    case "$sub" in
        load|get|show|cat|read|view|info|files|which|path|pin|unpin|validate)
            local skills=$(skills list -s 2>/dev/null)
            COMPREPLY=( $(compgen -W "$skills" -- "$cur") )
            return 0
            ;;
        ref|run)
            if [[ $cword -eq 2 ]]; then
                local skills=$(skills list -s 2>/dev/null)
                COMPREPLY=( $(compgen -W "$skills" -- "$cur") )
            fi
            return 0
            ;;
        completions)
            COMPREPLY=( $(compgen -W "fish bash zsh install" -- "$cur") )
            return 0
            ;;
    esac
}
complete -F _skills_completions skills
"#;

const ZSH_COMPLETIONS: &str = r#"#compdef skills
_skills() {
    local -a subcommands
    subcommands=(
        'list:List all available skills'
        'search:Search skills by keyword'
        'load:Load skill instructions into context'
        'get:Load skill instructions into context'
        'show:Show skill instructions'
        'cat:Cat skill instructions'
        'read:Read skill instructions'
        'view:View skill instructions'
        'info:Inspect deep skill metadata and token weights'
        'ref:Query and load reference documents'
        'run:Execute bundled scripts'
        'files:List all files in a skill'
        'stats:Global vault statistics'
        'config:Manage search directories'
        'skill:Print embedded meta-skill instructions'
        'which:Print absolute path to a skill or reference'
        'path:Print absolute path to a skill or reference'
        'pin:Symlink skill into agent directories'
        'unpin:Remove skill from agent directories'
        'pinned:Show currently pinned skills'
        'doctor:Audit vault and agent directory health'
        'validate:Validate skills against Agent Skills spec'
        'completions:Generate or install shell completions'
    )

    if (( CURRENT == 2 )); then
        _describe -t commands 'skills command' subcommands
        return
    fi

    case $words[2] in
        load|get|show|cat|read|view|info|files|which|path|pin|unpin|validate)
            local -a skill_names
            skill_names=($(skills list -s 2>/dev/null))
            _describe -t skills 'skill' skill_names
            ;;
        ref|run)
            if (( CURRENT == 3 )); then
                local -a skill_names
                skill_names=($(skills list -s 2>/dev/null))
                _describe -t skills 'skill' skill_names
            fi
            ;;
        completions)
            _values 'shell' fish bash zsh install
            ;;
    esac
}
_skills "$@"
"#;
