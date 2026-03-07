# Themes

Rimuru's TUI includes 15 themes inspired by Tensura (That Time I Got Reincarnated as a Slime) and popular editor themes.

## Tensura Themes (7)

| Theme | Style |
|-------|-------|
| **Rimuru Slime** | Blue-cyan slime aesthetic |
| **Great Sage** | Sage green analytical tones |
| **Predator** | Dark predatory reds |
| **Veldora** | Storm dragon golden-purple |
| **Shion** | Purple warrior energy |
| **Milim** | Pink destroyer vibrancy |
| **Diablo** | Dark crimson demon |

## Editor Themes (8)

| Theme | Style |
|-------|-------|
| **Tokyo Night** | Dark purple-blue |
| **Catppuccin** | Warm dark with pastels |
| **Dracula** | Classic dark purple |
| **Nord** | Arctic blue-gray |
| **Gruvbox** | Retro warm browns |
| **Rose Pine** | Soft dark with rose accents |
| **Cyberpunk** | Neon electric colors |
| **Tempest** | Storm-inspired dark theme |

## Switching Themes

### In TUI

Press `t` to cycle through all 15 themes.

### On Startup

```bash
rimuru-tui --theme "Great Sage"
rimuru-tui --theme "Tokyo Night"
```

## Theme Properties

Each theme defines colors for:

- `bg` / `fg` - Background and foreground
- `accent` - Highlight color
- `border` - Border color
- `success` / `warning` / `error` - Status colors
- `gauge_low` / `gauge_mid` / `gauge_high` - Gauge fill colors
- `selection_bg` / `selection_fg` - Selected item colors
