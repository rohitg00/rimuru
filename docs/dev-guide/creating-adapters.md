# Creating Agent Adapters

Adapters discover AI coding agents by reading their config directories on disk.

## Current Architecture

Adapters live in `crates/rimuru-core/src/adapters/`. Each adapter:

1. Checks if the agent is installed (config dir exists)
2. Reads session/cost data from the agent's local storage
3. Returns structured data that gets stored in iii-engine KV state

## Adding a New Agent

### Step 1: Add Agent Type

Edit `crates/rimuru-core/src/models/agent.rs`:

```rust
pub enum AgentType {
    ClaudeCode,
    Cursor,
    Copilot,
    Codex,
    Goose,
    OpenCode,
    MyAgent,  // Add here
}
```

Update `display_name()`, `icon()`, and `type_id()` match arms.

### Step 2: Create Adapter Module

Create `crates/rimuru-core/src/adapters/myagent.rs`:

```rust
use std::path::PathBuf;
use serde_json::json;

pub struct MyAgentAdapter;

impl MyAgentAdapter {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".myagent")
    }

    pub fn is_installed() -> bool {
        Self::config_dir().exists()
    }

    pub async fn discover_sessions() -> Vec<serde_json::Value> {
        let config_dir = Self::config_dir();
        if !config_dir.exists() {
            return vec![];
        }
        // Read session data from agent's local storage
        // (JSON files, SQLite, logs, etc.)
        vec![]
    }
}
```

### Step 3: Register in Adapters Module

Edit `crates/rimuru-core/src/adapters/mod.rs`:

```rust
pub mod myagent;
```

### Step 4: Add Discovery

Edit `crates/rimuru-core/src/functions/agents.rs` to include your adapter in the agent discovery function. The discovery function scans for installed agents and stores them in KV state.

### Step 5: Add Character Sprite (Optional)

To show your agent in the Tempest City view, add a sprite mapping in `ui/src/city/characters.ts`:

```typescript
const AGENT_TYPE_MAP: Record<string, CharacterType> = {
  claude_code: "rimuru",
  cursor: "shion",
  // ...
  my_agent: "gabiru",
};
```

## Testing

```bash
cargo test -p rimuru-core
```
