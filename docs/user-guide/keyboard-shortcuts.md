---
type: reference
title: Keyboard Shortcuts
created: 2026-02-05
tags:
  - keyboard
  - shortcuts
  - keybindings
  - tui
  - desktop
related:
  - "[[themes]]"
  - "[[getting-started]]"
---

# Keyboard Shortcuts

Complete reference for TUI and desktop application keybindings.

## TUI Keybindings

### Global Navigation

| Key | Alternative | Action |
|-----|-------------|--------|
| `q` | | Quit current view / application |
| `Ctrl+c` | | Force quit |
| `Tab` | | Next view |
| `Shift+Tab` | | Previous view |
| `1` | | Go to Dashboard |
| `2` | | Go to Agents |
| `3` | | Go to Sessions |
| `4` | | Go to Costs |
| `5` | | Go to Metrics |
| `?` | | Show help overlay |
| `t` | | Toggle theme |
| `r` | | Refresh current view |

### List Navigation

| Key | Alternative | Action |
|-----|-------------|--------|
| `j` | `↓` (Down) | Move down |
| `k` | `↑` (Up) | Move up |
| `h` | `←` (Left) | Move left / collapse |
| `l` | `→` (Right) | Move right / expand |
| `g` | | Go to top |
| `G` | `Shift+g` | Go to bottom |
| `Ctrl+u` | `Page Up` | Page up |
| `Ctrl+d` | `Page Down` | Page down |
| `Enter` | | Select / open details |
| `Esc` | | Back / cancel |

### Search and Filter

| Key | Action |
|-----|--------|
| `/` | Open search |
| `f` | Toggle filter panel |
| `s` | Cycle sort options |
| `Esc` | Close search / filter |

### Command Palette

| Key | Action |
|-----|--------|
| `:` | Open command palette |
| `Esc` | Close command palette |
| `Enter` | Execute command |
| `↑` / `↓` | Navigate commands |

### Help Modal

| Key | Action |
|-----|--------|
| `?` | Open help |
| `Esc` | Close help |
| `j` / `k` | Scroll help content |

## View-Specific Shortcuts

### Dashboard View

| Key | Action |
|-----|--------|
| `r` | Refresh all metrics |
| `Enter` | View selected item details |

### Agents View

| Key | Action |
|-----|--------|
| `Enter` | View agent details |
| `c` | Connect to agent |
| `d` | Disconnect agent |
| `/` | Search agents |
| `s` | Sort by column |

### Sessions View

| Key | Action |
|-----|--------|
| `Enter` | View session details |
| `a` | Toggle active only |
| `/` | Search sessions |
| `s` | Cycle sort (time, duration, tokens, cost) |
| `f` | Open filter panel |

### Costs View

| Key | Action |
|-----|--------|
| `Tab` | Cycle: Summary → By Agent → By Model |
| `r` | Change time range |
| `Enter` | View breakdown details |

### Metrics View

| Key | Action |
|-----|--------|
| `r` | Refresh metrics |
| `Tab` | Cycle metric panels |

## Dialog Shortcuts

### Confirmation Dialogs

| Key | Action |
|-----|--------|
| `y` | Confirm (Yes) |
| `n` | Cancel (No) |
| `Tab` | Toggle selection |
| `←` / `→` | Navigate buttons |
| `Enter` | Activate selected button |
| `Esc` | Cancel |

### Toast Notifications

| Key | Action |
|-----|--------|
| `Esc` | Dismiss current toast |

## Desktop Application Shortcuts

### Global (All Platforms)

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + D` | Go to Dashboard |
| `Cmd/Ctrl + A` | Go to Agents |
| `Cmd/Ctrl + S` | Go to Sessions |
| `Cmd/Ctrl + ,` | Open Settings |
| `Shift + ?` | Show keyboard shortcuts help |
| `Cmd/Ctrl + R` | Refresh |
| `Cmd/Ctrl + Q` | Quit application |

### macOS Specific

| Shortcut | Action |
|----------|--------|
| `Cmd + H` | Hide window |
| `Cmd + M` | Minimize window |
| `Cmd + W` | Close window |
| `Cmd + Q` | Quit |

### Windows/Linux Specific

| Shortcut | Action |
|----------|--------|
| `Alt + F4` | Close application |
| `F11` | Toggle fullscreen |

### Menu Bar Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + N` | New Agent |
| `Cmd/Ctrl + ,` | Settings |
| `Cmd/Ctrl + Shift + R` | Hard Refresh |
| `Cmd/Ctrl + F` | Focus Search |
| `F1` | Open Documentation |

## Customizing Keybindings

### Configuration File

Keybindings can be customized in:
- Linux/macOS: `~/.config/rimuru/keybinds.toml`
- Windows: `%APPDATA%\rimuru\keybinds.toml`

### Example Configuration

```toml
[quit]
code = "Char(q)"
modifiers = "None"

[[quit]]
code = "Char(c)"
modifiers = "Ctrl"

[next_view]
code = "Tab"
modifiers = "None"

[prev_view]
code = "BackTab"
modifiers = "None"

[up]
code = "Char(k)"
modifiers = "None"

[[up]]
code = "Up"
modifiers = "None"

[down]
code = "Char(j)"
modifiers = "None"

[[down]]
code = "Down"
modifiers = "None"

[select]
code = "Enter"
modifiers = "None"

[back]
code = "Esc"
modifiers = "None"

[search]
code = "Char(/)"
modifiers = "None"

[command]
code = "Char(:)"
modifiers = "None"

[toggle_theme]
code = "Char(t)"
modifiers = "None"

[help]
code = "Char(?)"
modifiers = "None"
```

### Key Code Reference

Available key codes:
- `Char(x)` - Single character (e.g., `Char(q)`, `Char(/)`)
- `Tab` - Tab key
- `BackTab` - Shift+Tab
- `Enter` - Enter/Return
- `Esc` - Escape
- `Up`, `Down`, `Left`, `Right` - Arrow keys
- `PageUp`, `PageDown` - Page navigation
- `Home`, `End` - Beginning/end
- `Delete`, `Backspace` - Delete keys
- `F1` through `F12` - Function keys

### Modifier Reference

Available modifiers:
- `None` - No modifier
- `Ctrl` - Control key
- `Alt` - Alt/Option key
- `Shift` - Shift key
- `Ctrl+Shift` - Combined modifiers
- `Ctrl+Alt` - Combined modifiers

## Vim-Style Navigation

Rimuru TUI supports Vim-style navigation by default:

| Vim Key | Action |
|---------|--------|
| `h` | Left |
| `j` | Down |
| `k` | Up |
| `l` | Right |
| `gg` | Go to top |
| `G` | Go to bottom |
| `/` | Search |
| `:` | Command mode |
| `Ctrl+u` | Half page up |
| `Ctrl+d` | Half page down |

## Quick Reference Card

```
╔════════════════════════════════════════════════════════════╗
║                 RIMURU TUI QUICK REFERENCE                 ║
╠════════════════════════════════════════════════════════════╣
║  NAVIGATION          │  ACTIONS                            ║
║  ─────────────────────┼─────────────────────────────────── ║
║  q        Quit        │  Enter    Select                   ║
║  Tab      Next view   │  Esc      Back                     ║
║  1-5      Go to view  │  r        Refresh                  ║
║  j/k/↑/↓  Move        │  t        Theme                    ║
║  g/G      Top/Bottom  │  ?        Help                     ║
║  Ctrl+u/d Page scroll │  /        Search                   ║
╠════════════════════════════════════════════════════════════╣
║  VIEWS: 1=Dashboard 2=Agents 3=Sessions 4=Costs 5=Metrics  ║
╚════════════════════════════════════════════════════════════╝
```

## Related Topics

- [[themes]] - Customizing appearance
- [[getting-started]] - Initial setup
