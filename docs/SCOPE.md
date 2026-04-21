# agent-cli

A single, agent-agnostic CLI for managing the context that AI coding agents load at runtime. Two concerns:

- **Skills** — rule/prompt repos cloned from GitHub and kept up to date via git.
- **MCP servers** — one `mcp-compose.yaml` synced to every agent config file on the machine.

The CLI is a package manager for agent context. It does not run agents.

---

## Skills

### How it works

A skill is a git repository that contains a single markdown file (conventionally `SKILL.md`) with all the rules an agent should follow. The CLI clones the repo locally; the agent loads from the path printed by `agent-cli path <name>`.

```
~/.agent-cli/
  skills/
    rust/            ← shallow git clone
      SKILL.md       ← what the agent loads
      rules/         ← individual rule files (for humans, not the agent)
    flutter/
      SKILL.md
  config.yaml        ← custom registries you've added
```

Individual rule files (e.g. `rules/async-try-join.md`) exist for human browsing. The agent loads only the single top-level skill file — loading 179 separate files would be information overload.

### Commands

```bash
agent-cli ls-remote                  # list all available skills (✓ = installed)
agent-cli install <name>             # git clone --depth=1
agent-cli update <name>              # git pull --ff-only
agent-cli update                     # update all installed skills
agent-cli remove <name>              # rm -rf
agent-cli ls                         # list installed skills + current commit SHA
agent-cli show <name>                # print the skill file content
agent-cli path <name>                # print the skill file path (use in agent config)
```

### Registries

Built-in registries are defined in [`registries.yaml`](../registries.yaml) at the repo root and compiled into the binary. To add a built-in, add an entry there and cut a release.

```yaml
registries:
  - name: rust
    url: https://github.com/leonardomso/rust-skills
    skill_file: SKILL.md
    description: 179 Rust rules for AI agents
```

Users can add their own registries. Any git repo with a top-level markdown file qualifies.

```bash
agent-cli registry add <name> <url>                    # default skill_file: SKILL.md
agent-cli registry add <name> <url> --skill-file RULES.md
agent-cli registry list                                # built-in + custom
agent-cli registry remove <name>                       # custom only; built-ins are immutable
```

Custom registries are saved to `~/.agent-cli/config.yaml`. A custom entry with the same name as a built-in overrides it.

---

## MCP server sync

Maintain one `mcp-compose.yaml` per repo, then sync it to every agent config file on the machine (Claude Desktop, Cursor, VS Code, Gemini, etc.).

### Commands

```bash
agent-cli init <stack>               # write a template mcp-compose.yaml
agent-cli sync                       # sync mcp-compose.yaml to all agent configs
agent-cli health                     # test connectivity of all servers
```

Available stacks: `rust`, `frontend` (aliases: `typescript`, `react`), `shadcn` (alias: `shadcn-ui`).

### mcp-compose.yaml format

```yaml
agents:
  - name: claude
    path: "~/.config/Claude/claude_desktop_config.json"
    format: standard
  - name: cursor
    path: ".cursor/mcp.json"
    format: standard
  - name: vscode
    path: ".vscode/settings.json"
    format: vscode

servers:
  context7:
    command: npx
    args: ["-y", "@upstash/context7-mcp"]
  github:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_PERSONAL_ACCESS_TOKEN: "set-me"
```

---

## Design decisions

**One file per skill.** The agent loads a single markdown file. Splitting rules across hundreds of files adds latency and noise. The `SKILL.md` convention (established by the leonardomso/rust-skills ecosystem) is the standard this CLI follows.

**Git as the transport.** Using `git clone` + `git pull` gives versioning, offline use, and diff history for free. No custom lock file or sha256 logic needed — git history provides provenance.

**Built-ins are immutable.** `registry remove` only works on custom registries. Built-ins ship in the binary; to override one, `registry add` an entry with the same name pointing at a different URL.

**No agent coupling.** The CLI writes to `~/.agent-cli/skills/`. How agents load from there is up to the user (symlink, config entry, or `agent-cli path` in a script). Future `init <agent>` subcommands will handle wiring automatically.
