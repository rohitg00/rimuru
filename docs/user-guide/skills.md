---
type: reference
title: Using SkillKit Integration
created: 2026-02-05
tags:
  - skills
  - skillkit
  - marketplace
  - agents
related:
  - "[[agents]]"
  - "[[plugins]]"
  - "[[cli-reference]]"
---

# Using SkillKit Integration

Rimuru integrates with SkillKit, a universal skill management system for AI coding agents. Install, translate, and manage skills across multiple agents from a single interface.

## Prerequisites

SkillKit must be installed to use skills:

```bash
npm i -g skillkit
```

Or use via npx:

```bash
npx skillkit --help
```

Rimuru will detect if SkillKit is available and guide you to install it if needed.

## CLI Commands

### Search Skills

Search the SkillKit marketplace:

```bash
rimuru skills search <query>
```

Example:

```bash
rimuru skills search "react hooks"
```

Output:

```
Searching marketplace for: react hooks

┌───────────────────────┬────────────────────────────────────────┬────────────┬────────────────┬───────────┐
│ Name                  │ Description                            │ Author     │ Tags           │ Downloads │
├───────────────────────┼────────────────────────────────────────┼────────────┼────────────────┼───────────┤
│ react-useeffect       │ React useEffect best practices         │ skillkit   │ react, hooks   │ 15.2K     │
│ react-hooks-patterns  │ Advanced React hooks patterns          │ devtools   │ react, advanced│ 8.7K      │
│ react-state-mgmt      │ State management patterns for React    │ community  │ react, state   │ 6.3K      │
└───────────────────────┴────────────────────────────────────────┴────────────┴────────────────┴───────────┘

  Found 23 skills (showing 20)

Use 'rimuru skills show <name>' to view skill details.
```

Filter by agent:

```bash
rimuru skills search "testing" --agent claude-code
```

Filter by tags:

```bash
rimuru skills search "api" --tags "typescript,rest"
```

Limit results:

```bash
rimuru skills search "database" --limit 10
```

### Install Skills

Install a skill for an agent:

```bash
rimuru skills install <skill-name>
```

Specify target agent:

```bash
rimuru skills install react-useeffect --agent claude-code
```

Install for all supported agents:

```bash
rimuru skills install react-useeffect --all
```

### List Installed Skills

View all installed skills:

```bash
rimuru skills list
```

Filter by agent:

```bash
rimuru skills list --agent cursor
```

Show only enabled skills:

```bash
rimuru skills list --enabled
```

Show only disabled skills:

```bash
rimuru skills list --disabled
```

### Show Skill Details

View detailed information about a skill:

```bash
rimuru skills show <skill-name>
```

Displays:
- Name and slug
- Description
- Author and source
- Version
- Download count
- Tags
- Compatible agents
- Created/updated dates

### Translate Skills

Translate a skill from one agent format to another:

```bash
rimuru skills translate <skill-name> --from claude-code --to cursor
```

Example:

```
Translating 'react-useeffect' from Claude Code to Cursor...

✓ Skill 'react-useeffect' translated successfully!

  Translation:   Claude Code → Cursor
  Output:        ~/.cursor/skills/react-useeffect/SKILL.md
  Duration:      234ms

  Warnings:
    ⚠ Some agent-specific features may need manual adjustment
```

### Uninstall Skills

Remove an installed skill:

```bash
rimuru skills uninstall <skill-name>
```

### Get Recommendations

Get AI-powered skill recommendations:

```bash
rimuru skills recommend
```

Get recommendations based on workflow:

```bash
rimuru skills recommend --workflow "building a REST API with Express"
```

Output:

```
Skill Recommendations

  → Based on workflow: building a REST API with Express

┌───┬────────────────────────┬─────────────────────────────────────────────┬────────────┐
│ # │ Skill                  │ Reason                                      │ Confidence │
├───┼────────────────────────┼─────────────────────────────────────────────┼────────────┤
│ 1 │ express-routing        │ Core Express patterns for REST endpoints    │ 95%        │
│ 2 │ api-validation         │ Request validation and error handling       │ 88%        │
│ 3 │ nodejs-security        │ Security best practices for Node.js APIs   │ 85%        │
│ 4 │ typescript-patterns    │ Type-safe API development patterns         │ 82%        │
│ 5 │ database-schema        │ Database design for REST APIs              │ 78%        │
└───┴────────────────────────┴─────────────────────────────────────────────┴────────────┘

Use 'rimuru skills install <name>' to install a recommended skill.
```

### Publish Skills

Publish a skill to the marketplace:

```bash
rimuru skills publish ./path/to/SKILL.md
```

With validation:

```bash
rimuru skills publish ./SKILL.md
```

Skip validation (not recommended):

```bash
rimuru skills publish ./SKILL.md --skip-validation
```

### Sync with Marketplace

Synchronize local skills with the marketplace:

```bash
rimuru skills sync
```

### View Marketplace Stats

View SkillKit marketplace statistics:

```bash
rimuru skills stats
```

Output:

```
SkillKit Marketplace Statistics
══════════════════════════════════════════════════════

  Overview
    Total Skills:         15,062
    Total Downloads:     2.3M
    Total Authors:           891

  Skills by Agent
    Claude Code              4,521
    Cursor                   3,892
    Codex                    2,145
    Goose                    1,876
    OpenCode                 1,234
    ...

  Trending Skills
    • pro-workflow
    • react-useeffect
    • typescript-patterns
    • security-review
    • tdd-workflow

  Last Updated: 2026-02-05 14:30:00 UTC
```

### List Supported Agents

View all agents supported by SkillKit:

```bash
rimuru skills agents
```

Output:

```
Supported Agents
══════════════════════════════════════════════════════

┌──────────────────┬─────────────────┬──────┐
│ ID               │ Name            │ Icon │
├──────────────────┼─────────────────┼──────┤
│ claude-code      │ Claude Code     │ ⟁    │
│ cursor           │ Cursor          │ ◫    │
│ codex            │ Codex           │ ◎    │
│ gemini-cli       │ Gemini CLI      │ ✦    │
│ opencode         │ OpenCode        │ ◇    │
│ github-copilot   │ GitHub Copilot  │ ◈    │
│ goose            │ Goose           │ ⬡    │
│ windsurf         │ Windsurf        │ ⌘    │
│ ...              │ ...             │ ...  │
└──────────────────┴─────────────────┴──────┘

  Total: 32 agents supported
```

## Skill File Format

Skills are defined in `SKILL.md` files:

```markdown
---
name: my-skill
version: 1.0.0
description: A helpful skill
author: your-name
tags:
  - typescript
  - testing
agents:
  - claude-code
  - cursor
---

# My Skill

Skill content here...

## When to Use

- Scenario 1
- Scenario 2

## Instructions

1. Step one
2. Step two
```

## TUI Skills View

In the TUI, press `6` (if available) to access the Skills view:

```
┌─ Skills ────────────────────────────────────────────────────┐
│ Installed Skills                                            │
├─────────────────────────────────────────────────────────────┤
│  Name                  Agents           Status    Installed │
│  react-useeffect       ⟁ ◫ ◎          ✓ Enabled  2026-02-01│
│  typescript-patterns   ⟁ ◫            ✓ Enabled  2026-01-28│
│  security-review       ⟁              ✓ Enabled  2026-01-25│
│  pro-workflow          ⟁ ◫ ◎ ◇ ⬡    ✓ Enabled  2026-01-20│
└─────────────────────────────────────────────────────────────┘
```

Navigation:
- `j/k`: Navigate skills list
- `Enter`: View skill details
- `i`: Install new skill
- `u`: Uninstall selected skill
- `/`: Search installed skills

## JSON Output

All commands support JSON output for scripting:

```bash
rimuru skills search "react" --format json
rimuru skills list --format json
rimuru skills show react-useeffect --format json
rimuru skills stats --format json
rimuru skills agents --format json
```

## Configuration

Configure SkillKit integration in `config/local.toml`:

```toml
[skillkit]
default_agent = "claude-code"
auto_install_for_all = false
marketplace_url = "https://marketplace.skillkit.dev"
```

## Related Topics

- [[agents]] - Agent management
- [[plugins]] - Plugin system
- [[cli-reference]] - Complete CLI reference
