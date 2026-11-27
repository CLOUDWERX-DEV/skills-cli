# Security

## What `skills` runs, and as whom

Most of what `skills` does is read files. Two commands do more:

- **`skills run`** executes a script from a skill's `scripts/` folder, with your permissions, exactly as if you ran it yourself. There's no sandbox.
- **`skills install`** copies a skill's files into your library. It doesn't run anything, but it puts scripts where `skills run` (and your agent) can reach them.

So treat an installed skill like any other code you download: read its `SKILL.md` and scripts before you trust it, and only install from sources you trust. An agent that can run `skills run` can run any script in your library.

## What it protects against

- **Path tricks.** Reference and script paths can't leave their skill folder (`../../etc/passwd` is refused), and `install --path` can't leave the source.
- **Deleting through links.** `remove` and `unpin` remove a link, never the folder it points at. `remove` does nothing without `--yes`.
- **Overwriting by accident.** `install` and `new` leave existing skills alone unless you pass `--force`.
- **Context floods.** Loading every reference of a skill is capped by a token limit unless you pass `--force`.
- **Copying repository history.** `install` never copies `.git`, and it skips links to folders.

## Reporting a problem

Please don't open a public issue for a security problem. Use a [private security advisory](https://github.com/CLOUDWERX-DEV/skills-cli/security/advisories/new) or email mail@cloudwerxlab.com. I'll answer as soon as I can.
