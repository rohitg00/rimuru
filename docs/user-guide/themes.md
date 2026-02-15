---
type: reference
title: Customizing Appearance
created: 2026-02-05
tags:
  - themes
  - customization
  - tui
  - appearance
related:
  - "[[keyboard-shortcuts]]"
  - "[[getting-started]]"
---

# Customizing Appearance

Rimuru's TUI supports multiple themes and customization options. Choose from built-in themes or create your own.

## Built-in Themes

Rimuru includes 7 built-in themes:

| Theme | Description | Style |
|-------|-------------|-------|
| **Tokyo Night** | Default dark theme | Dark purple/blue with subtle highlights |
| **Catppuccin Mocha** | Warm dark theme | Dark with pastel accents |
| **Catppuccin Latte** | Light theme | Light with warm colors |
| **Dracula** | Classic dark theme | Dark purple with vibrant colors |
| **Nord** | Arctic-inspired | Blue-gray palette |
| **Gruvbox Dark** | Retro dark theme | Warm browns and oranges |
| **Gruvbox Light** | Retro light theme | Light with warm accents |

## Changing Themes

### In TUI

Press `t` to cycle through available themes.

### Via Configuration

Set your preferred theme in `config/local.toml`:

```toml
[tui]
theme = "catppuccin-mocha"
```

Available values:
- `tokyo-night` (default)
- `catppuccin-mocha`
- `catppuccin-latte`
- `dracula`
- `nord`
- `gruvbox-dark`
- `gruvbox-light`

## Theme Previews

### Tokyo Night (Default)

```
┌─────────────────────────────────────────────────┐
│  ██████  Background: #1a1b26                    │
│  ██████  Foreground: #c0caf5                    │
│  ██████  Accent:     #7aa2f7                    │
│  ██████  Success:    #9ece6a                    │
│  ██████  Warning:    #e0af68                    │
│  ██████  Error:      #f7768e                    │
└─────────────────────────────────────────────────┘
```

### Catppuccin Mocha

```
┌─────────────────────────────────────────────────┐
│  ██████  Background: #1e1e2e                    │
│  ██████  Foreground: #cdd6f4                    │
│  ██████  Accent:     #89b4fa                    │
│  ██████  Success:    #a6e3a1                    │
│  ██████  Warning:    #f9e2af                    │
│  ██████  Error:      #f38ba8                    │
└─────────────────────────────────────────────────┘
```

### Dracula

```
┌─────────────────────────────────────────────────┐
│  ██████  Background: #282a36                    │
│  ██████  Foreground: #f8f8f2                    │
│  ██████  Accent:     #bd93f9                    │
│  ██████  Success:    #50fa7b                    │
│  ██████  Warning:    #ffb86c                    │
│  ██████  Error:      #ff5555                    │
└─────────────────────────────────────────────────┘
```

### Nord

```
┌─────────────────────────────────────────────────┐
│  ██████  Background: #2e3440                    │
│  ██████  Foreground: #eceff4                    │
│  ██████  Accent:     #88c0d0                    │
│  ██████  Success:    #a3be8c                    │
│  ██████  Warning:    #ebcb8b                    │
│  ██████  Error:      #bf616a                    │
└─────────────────────────────────────────────────┘
```

## Theme Components

Each theme defines colors for:

| Component | Usage |
|-----------|-------|
| `background` | Main background color |
| `foreground` | Primary text color |
| `foreground_dim` | Secondary/muted text |
| `surface` | Panel and card backgrounds |
| `border` | Borders and dividers |
| `selection` | Selected item highlight |
| `accent` | Primary accent color |
| `accent_secondary` | Secondary accent |
| `success` | Success indicators |
| `warning` | Warning indicators |
| `error` | Error indicators |
| `info` | Information indicators |

## Custom Themes

Create custom themes by adding a theme file:

Location: `~/.config/rimuru/themes/`

### Theme File Format

Create `my-theme.toml`:

```toml
[theme]
name = "my-theme"

[colors]
background = "#1e1e2e"
foreground = "#cdd6f4"
foreground_dim = "#6c7086"
surface = "#313244"
border = "#45475a"
selection = "#45475a"
accent = "#89b4fa"
accent_secondary = "#cba6f7"
success = "#a6e3a1"
warning = "#f9e2af"
error = "#f38ba8"
info = "#89dceb"
```

### Using Custom Themes

After creating the theme file:

```toml
[tui]
theme = "my-theme"
```

Or in TUI, press `t` to cycle to your custom theme.

## CLI Color Output

### Enable/Disable Colors

Environment variable:

```bash
export RIMURU_COLOR=true   # Enable colors
export RIMURU_COLOR=false  # Disable colors
```

In configuration:

```toml
[display]
color = true
```

### Force Colors

For piping output that should retain colors:

```bash
RIMURU_COLOR=true rimuru agents | less -R
```

## Terminal Compatibility

Rimuru works best with terminals that support:

- **256 colors** or **True Color (24-bit)**
- **Unicode** characters for icons and borders

### Recommended Terminals

| Terminal | Platform | True Color | Notes |
|----------|----------|------------|-------|
| iTerm2 | macOS | Yes | Recommended for macOS |
| Alacritty | All | Yes | Fast GPU-accelerated |
| Kitty | All | Yes | Feature-rich |
| WezTerm | All | Yes | Modern, configurable |
| Windows Terminal | Windows | Yes | Recommended for Windows |
| GNOME Terminal | Linux | Yes | Good default |
| VS Code Terminal | All | Yes | Works well |

### Limited Color Terminals

If using a terminal with limited colors, set:

```toml
[tui]
true_color = false
```

This falls back to 256-color mode.

## Font Recommendations

For best icon rendering, use a Nerd Font:

- **JetBrains Mono Nerd Font**
- **FiraCode Nerd Font**
- **Hack Nerd Font**
- **Iosevka Nerd Font**

Download from: https://www.nerdfonts.com/

## Accessibility

### High Contrast

For better visibility, use themes with high contrast:

1. **Catppuccin Latte** (light theme)
2. **Gruvbox Light** (light theme)
3. **Dracula** (dark theme with vibrant colors)

### Reduced Motion

Disable animations in configuration:

```toml
[tui]
animations = false
```

### Screen Reader Compatibility

While Rimuru's TUI is visual, you can use CLI commands with JSON output for screen reader compatibility:

```bash
rimuru status --format json
rimuru agents --format json
```

## Desktop Application Themes

The desktop application (`rimuru-desktop`) follows system theme settings:

- **macOS**: Follows system dark/light mode
- **Windows**: Follows system theme
- **Linux**: Follows desktop environment theme

Override in desktop settings if needed.

## Troubleshooting

### Colors Look Wrong

1. Check terminal True Color support
2. Verify `TERM` environment variable
3. Try forcing True Color: `export COLORTERM=truecolor`

### Icons Missing

1. Install a Nerd Font
2. Configure terminal to use the font
3. Check Unicode support in terminal

### Theme Not Loading

1. Verify theme file syntax
2. Check file permissions
3. Review logs: `RIMURU_LOG_LEVEL=debug rimuru-tui`

## Related Topics

- [[keyboard-shortcuts]] - TUI navigation and keybinds
- [[getting-started]] - Initial setup
