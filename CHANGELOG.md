# Changelog

All notable changes to `skills` are in this file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- **A new logo**: an SD card with four green contacts and the word "skills", in the icon and the banner (`assets/`). The vector mark is `assets/src/logo.svg`, traced from the artwork in `assets/src/logo-master.png`.
- The pixel-art logo generator (`examples/make_logo.rs`) is gone; the icon and banner are plain files in `assets/`.


## [0.2.0] - 2026-11-26

First public release.

### Added

- **`skills new NAME`** scaffolds a valid skill (a `SKILL.md`, `references/` and `scripts/`) to fill in. It refuses bad names and never overwrites.
- **`skills install SOURCE`** installs skills from a git URL, `owner/repo` on GitHub, or a local folder. It finds one skill or many, and takes `--skill` (pick some), `--path` (look in a subfolder), `--to` (choose the library folder) and `--force` (replace). It never copies `.git`, skips links to folders, and skips skills with no valid name or description.
- **`skills remove NAME --yes`** removes a skill. Without `--yes` it only says what it would do, and it removes a link, never what the link points at.
- `skills load`, `get`, `show` and `cat` with single and multi-skill loading, token counts and `--deps` for dependencies.
- `skills list` (cards, `--table`, `--short`, `--full`, `--json`), `skills search` (ranked across names, descriptions and compatibility), `skills info`, `skills files` and `skills stats`.
- `skills ref` to list, search and fetch reference docs with fuzzy names, in verb-first and skill-first forms. `ref all` is guarded by a token limit (`--force` lifts it).
- `skills run` executes bundled scripts (Python, Bash, Fish, Node or a binary) and only the output comes back.
- `skills which` prints the path of a skill, reference or script.
- `skills pin`, `unpin` and `pinned` link skills into Claude Code, Codex, Antigravity, OpenCode, Pi, Hermes and the shared agents folder, so they show up as slash commands.
- `skills doctor [--fix]` finds and repairs dead links, rogue folders and script permissions. With no meta-skill in the library, `--fix` writes the one built into the binary.
- `skills validate` checks skills against the skill format, including name and description limits, link integrity and script shebangs.
- `skills config show|path|init|add`, with search order `-d`, `$SKILLS_PATH`, project folders, the config file, then the defaults. The config file is `~/.config/skills-cli/config.toml`.
- `skills completions` for fish, bash and zsh, with dynamic skill names and `--install`.
- `skills skill` prints a guide built into the binary, so an agent can learn the tool with one command. The same guide is a skill in `skills/skills-cli`.
- Colors only on a terminal, a clean exit when piped into a pager (SIGPIPE), and no ANSI in piped output.
- A pixel-art mascot and banner, CI, release binaries for x86_64 and aarch64, and a PKGBUILD.
- Tests: unit tests plus end-to-end tests that run the real binary against a throwaway library and home folder, including path traversal checks.

### Changed

- Discovery skips hidden folders (like an installer's `.system`) and folders that have no `SKILL.md`, so they don't show up as empty skills.
- The config file moved to `~/.config/skills-cli/config.toml`, and the only environment variable is `SKILLS_PATH`.
- The default search folders are `~/.agents/skills`, `~/.claude/skills`, `~/.codex/skills` and `~/.gemini/antigravity-cli/skills`.
- The package is `skills-cli` and the command is still `skills`.

### Fixed

- `doctor` no longer reports every skill as a rogue folder when an agent's folder is the library reached through a link, and no longer links the meta-skill into itself.
- `pin`, `unpin`, `unpin --all` and `pinned` skip such a folder, so `unpin --all` can never delete links inside the library.
- `doctor` reports a dead link even under a system name.

[0.2.0]: https://github.com/CLOUDWERX-DEV/skills-cli/releases/tag/v0.2.0
