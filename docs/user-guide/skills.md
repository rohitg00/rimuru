# SkillKit Integration

Rimuru integrates with [SkillKit](https://agenstskills.com) for managing AI agent skills.

## Viewing Skills

### Web UI

The SkillKit page in the Web UI lets you browse and manage skills from the marketplace.

### API

```bash
curl http://localhost:3100/api/skillkit/skills
curl http://localhost:3100/api/skillkit/search?q=testing
```

## What Are Skills?

Skills are reusable instructions and workflows for AI coding agents. They can be installed across 32+ supported agents including Claude Code, Cursor, Codex, and more.

## SkillKit CLI

If you have SkillKit installed separately:

```bash
npx skillkit search "testing"
npx skillkit install pro-workflow
npx skillkit translate pro-workflow --agent cursor
```

## Bridge

Rimuru's SkillKit bridge (in `crates/rimuru-core/src/functions/skillkit.rs`) provides:

- Skill search and discovery
- Installation tracking
- Cross-agent translation
- Marketplace browsing
