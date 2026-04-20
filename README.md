# agent-cli

`agent-cli` bootstraps an AI coding setup on any developer machine.

It has two jobs:

- sync one `mcp-compose.yaml` into multiple agent configs
- install stack-specific markdown skills into `./docs/agents/`

## Commands

```bash
agent-cli init rust
agent-cli init react
agent-cli sync
agent-cli test
agent-cli skills list
agent-cli skills add rust typescript react shadcn
agent-cli skills installed
agent-cli skills update
agent-cli skills remove shadcn
```

## Registry

By default the CLI reads the remote registry from:

`https://raw.githubusercontent.com/NubeDev/agent-cli/master/skills-registry.yaml`

Override it with `--registry-url` or `AGENT_CLI_REGISTRY`.

Source skills are organized as `skills/<name>/README.md` so each skill can grow into its own folder when it needs extra notes, references, or assets.
