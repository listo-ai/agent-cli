# agent-cli

Agent-agnostic CLI for managing AI coding agent skills and MCP server config.

Two jobs:
- **Skills** — clone community skill repos into `~/.agent-cli/skills/` so any agent can load them.
- **MCP sync** — drop a `mcp-compose.yaml` template into a repo and sync it to every agent config file.

## Install

```bash
cargo install --path .
```

---

## Skills

Skills are git repos containing a `SKILL.md` (and optionally individual rule files).
The CLI clones them locally — agents load from the path printed by `agent-cli path <name>`.

```bash
# See what's available (built-in + any custom registries you've added)
agent-cli ls-remote

# Install a skill
agent-cli install rust
agent-cli install flutter

# List installed skills
agent-cli ls

# Update one skill
agent-cli update rust

# Update all installed skills
agent-cli update

# Print the skill file path (use this in agent config)
agent-cli path rust

# Print the skill content
agent-cli show rust

# Remove a skill
agent-cli remove rust
```

### Custom registries

Any git repo with a `SKILL.md` (or any markdown file) can be a skill source.

```bash
# Add a custom skill registry
agent-cli registry add myskill https://github.com/me/my-skills

# With a non-default skill file
agent-cli registry add myskill https://github.com/me/my-skills --skill-file RULES.md

# List all registries (built-in + custom)
agent-cli registry list

# Remove a custom registry (built-ins cannot be removed)
agent-cli registry remove myskill
```

Custom registries are saved to `~/.agent-cli/config.yaml`.
Built-in registries are defined in [`registries.yaml`](./registries.yaml) and shipped inside the binary.

### Built-in registries

See [`registries.yaml`](./registries.yaml) for the current list.

---

## MCP server sync

Drop a template `mcp-compose.yaml` into a repo, edit it, then sync to all agents.

```bash
# Write a template for your stack
agent-cli init rust
agent-cli init frontend
agent-cli init shadcn

# Sync mcp-compose.yaml to all configured agents
agent-cli sync

# Test health of all servers in mcp-compose.yaml
agent-cli health
```

Available templates: `rust`, `frontend` (alias: `typescript`, `react`), `shadcn` (alias: `shadcn-ui`).

---

## Skills directory

```
~/.agent-cli/
  skills/
    rust/          ← git clone of leonardomso/rust-skills
      SKILL.md     ← the file your agent loads
      rules/       ← individual rule files (for human browsing)
    flutter/       ← git clone of flutter skills repo
      SKILL.md
  config.yaml      ← custom registries you've added
```
