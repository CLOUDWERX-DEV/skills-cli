# The skill format

A skill is a folder with a `SKILL.md` in it. That's all `skills` needs, and it's the same format other agent tools read.

```
pdf-tools/
  SKILL.md          required: frontmatter, then instructions
  references/       optional: longer docs, loaded one at a time
  scripts/          optional: programs the agent runs instead of reading
  assets/           optional: templates, images, data
```

## SKILL.md

```markdown
---
name: pdf-tools
description: Read, merge and split PDF files. Use when a task involves a PDF, a form, or extracting text from one.
license: MIT
compatibility: Needs qpdf on PATH.
dependencies: [file-conversion]
---

# pdf tools

What it is, when to use it, the steps. Link to longer material:
see [merging](references/merging.md).
```

| Field | Required | Notes |
| --- | --- | --- |
| `name` | yes | Lowercase letters, digits and single hyphens, at most 64 characters. Should match the folder name. |
| `description` | yes | At most 1024 characters. This is what decides whether an agent reaches for the skill, so say what it does **and when to use it**. |
| `license` | no | |
| `compatibility` | no | Tools or environments it needs. It's searched too. |
| `allowed-tools` | no | A list of tools the skill expects to use. |
| `dependencies` | no | Other skills. `skills load --deps` pulls them in. |
| `metadata` | no | Free-form key and value pairs. |

## References

Keep `SKILL.md` short and move long material into `references/`, linked with relative paths. `skills validate` checks every such link points at a real file. Fetch them one at a time:

```sh
skills ref list pdf-tools
skills ref get pdf-tools merging       # fuzzy: "merging" finds references/guide/merging.md
```

## Scripts

Put programs in `scripts/` with a shebang (`#!/usr/bin/env python3`). `skills run pdf-tools split.py a.pdf` runs them, handling Python, Bash, Fish, Node and plain binaries, and only the output goes into context. `skills validate` checks the shebangs, and `skills doctor --fix` makes them executable.

## Where skills are found

1. `-d DIR` flags
2. `$SKILLS_PATH`, colon separated
3. `./.agents/skills` and `./.claude/skills` in the current folder
4. the config file, `~/.config/skills-cli/config.toml`
5. the defaults: `~/.agents/skills`, `~/.claude/skills`, `~/.codex/skills`, `~/.gemini/antigravity-cli/skills`

Hidden folders (starting with a dot) and folders without a `SKILL.md` are not skills. The first skill with a given name wins, so a project's own skills override your global ones.

## Pinning

`skills pin NAME --agents claude` makes a symlink in that agent's skills folder so the skill appears as a slash command. Pinning only ever adds and removes links, never the skill itself. If an agent's folder is really your library (reached through a link), `pin`, `unpin` and `doctor` leave it alone.
