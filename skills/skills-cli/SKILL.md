---
name: skills-cli
description: Find, load and manage agent skills on demand with the skills CLI instead of keeping them all in context. Use when a task falls in a domain you may have a skill for, when the user asks you to use or load a named skill, when you need a skill's reference docs or bundled scripts without dumping them whole, or when you need to create, install, validate or pin skills.
license: MIT
---

# skills

`skills` keeps a whole library of skills on disk and loads only what the task needs. Your startup context stays small and you pull in the details when they matter. Everything below runs in the shell.

## The rules

1. **Look before you guess.** If a task is in a domain a skill might cover, search first, then load it. Never invent an API a skill documents.
   ```sh
   skills search <words>
   skills load <skill>
   ```
2. **Load only what you need.** Several skills at once is fine, and `--deps` also loads anything they depend on.
   ```sh
   skills load pdf docx --deps
   ```
3. **Don't dump reference folders.** List or search them, then fetch one page.
   ```sh
   skills ref list <skill>
   skills ref search <skill> "<query>"
   skills ref get <skill> <file-or-topic>
   ```
   `skills ref all <skill>` is guarded by a token limit on purpose. Don't use `--force` unless the user asked for everything.
4. **Run bundled scripts, don't read them.** Only the output comes into context.
   ```sh
   skills run <skill> <script> [args...]
   ```
5. **Slash commands.** If the user wants a skill as a slash command in an agent, pin it. `unpin` removes it again.
   ```sh
   skills pin <skill> --agents claude,codex
   skills pinned
   skills unpin --all
   ```

## Commands

| Command | What it does |
| --- | --- |
| `skills list [--table\|--short\|--json] [filter]` | list the library |
| `skills search <query>` | ranked search over names, descriptions and compatibility |
| `skills load <skill...> [--deps]` | print a skill's instructions (aliases: `get`, `show`, `cat`) |
| `skills info <skill>` | structure, files and token weight |
| `skills ref <list\|get\|search\|all> <skill> ...` | work with a skill's reference docs |
| `skills run <skill> <script> [args]` | run a script from the skill's `scripts/` |
| `skills files <skill>` / `skills stats` | what's in one skill / in the whole library |
| `skills which <skill> [--ref F\|--script S]` | the path on disk |
| `skills new <name>` | scaffold a valid skill to fill in |
| `skills install <source>` | install skills from a git URL, `owner/repo` or a folder (`--path`, `--skill`, `--force`) |
| `skills remove <skill> --yes` | remove a skill (without `--yes` it only says what it would do) |
| `skills validate [skill...]` | check skills against the Agent Skills format |
| `skills doctor [--fix]` | find and repair broken links, rogue folders and script permissions |
| `skills pin` / `unpin` / `pinned` | link skills into agents' folders |
| `skills config show\|path\|init\|add <dir>` | where skills are searched |
| `skills completions install` | shell completions |
| `skills skill` | print this guide |

## Writing a good skill

- `skills new my-skill` gives you a valid start. The **description** is what decides whether it gets used: say what it does and when to reach for it, in under 1024 characters.
- Keep `SKILL.md` short. Put long material in `references/` and link to it with a relative path. `skills validate` checks every link points at a real file.
- Put helper programs in `scripts/` with a shebang and run them with `skills run`.
- Run `skills validate` before you share it.

## Where skills live

Search order: `-d DIR` flags, `$SKILLS_PATH` (colon separated), `./.agents/skills` and `./.claude/skills` in the current folder, then the config file's folders (default `~/.agents/skills`, `~/.claude/skills`, `~/.codex/skills`). `skills config show` prints what it resolved.
