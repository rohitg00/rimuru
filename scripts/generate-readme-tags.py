#!/usr/bin/env python3
"""
Tempest UI -- Rimuru README design system.

Generates Tensura-inspired "Unique Skill Notice" SVGs for the README.
Aesthetic: anime system-window frames with angular L-corner brackets,
diagonal stripe texture overlays, cyan-glow titles on indigo night-sky
gradients, and a Tempest Crest glyph that appears on every section header.

Tag categories
--------------
  section-*   skill notice headers with crest + title + notice id + rank
  stat-*      stat cards with italic Georgia numerals and accent underline
  pill-*      hexagonal skill tags with diamond indicator, rarity colors
  divider     slime-blob ornament between horizontal gradient lines
  hero        the banner at the top of the README

Output
------
  docs/assets/rimuru-banner.svg        (hero, single file)
  docs/assets/tags/*.svg               (dark variants, default in README)
  docs/assets/tags/light/*.svg         (light variants, shown on dark themes)

Run: python3 scripts/generate-readme-tags.py
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS_DIR = ROOT / "docs" / "assets"
DARK_DIR = ASSETS_DIR / "tags"
LIGHT_DIR = DARK_DIR / "light"
BANNER_PATH = ASSETS_DIR / "rimuru-banner.svg"

SERIF = "Georgia, 'Crimson Text', 'Times New Roman', serif"
MONO = "'SF Mono', Menlo, Consolas, 'Courier New', monospace"


@dataclass
class Palette:
    bg_a: str        # background gradient start
    bg_b: str        # background gradient end
    border: str      # frame border
    title: str       # primary text
    subtitle: str    # secondary text
    accent: str      # cyan base
    glow: str        # brighter cyan
    purple: str      # Rimuru purple
    gold: str        # legendary rank
    shimmer: str     # title highlight
    common: str      # rarity: common
    rare: str        # rarity: rare
    unique: str      # rarity: unique
    stripe_op: str   # stripe opacity
    fade_op: str     # gradient fade opacity


DARK = Palette(
    bg_a="#0A0E1A",
    bg_b="#141B2E",
    border="#1E293B",
    title="#F8FAFC",
    subtitle="#94A3B8",
    accent="#06B6D4",
    glow="#22D3EE",
    purple="#A78BFA",
    gold="#FBBF24",
    shimmer="#A5F3FC",
    common="#64748B",
    rare="#3B82F6",
    unique="#06B6D4",
    stripe_op="0.06",
    fade_op="0.9",
)

LIGHT = Palette(
    bg_a="#F8FAFC",
    bg_b="#E0F2FE",
    border="#CBD5E1",
    title="#0F172A",
    subtitle="#475569",
    accent="#0891B2",
    glow="#06B6D4",
    purple="#7C3AED",
    gold="#D97706",
    shimmer="#0369A1",
    common="#64748B",
    rare="#2563EB",
    unique="#0891B2",
    stripe_op="0.05",
    fade_op="0.75",
)


def _defs(pal: Palette) -> str:
    return f"""  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="{pal.bg_a}"/>
      <stop offset="1" stop-color="{pal.bg_b}"/>
    </linearGradient>
    <linearGradient id="titleGrad" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="{pal.title}"/>
      <stop offset="1" stop-color="{pal.shimmer}"/>
    </linearGradient>
    <linearGradient id="shim" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="{pal.accent}" stop-opacity="0"/>
      <stop offset="0.5" stop-color="{pal.glow}" stop-opacity="{pal.fade_op}"/>
      <stop offset="1" stop-color="{pal.accent}" stop-opacity="0"/>
    </linearGradient>
    <pattern id="grid" patternUnits="userSpaceOnUse" width="10" height="10" patternTransform="rotate(45)">
      <line x1="0" y1="0" x2="0" y2="10" stroke="{pal.accent}" stroke-width="1.4" opacity="{pal.stripe_op}"/>
    </pattern>
  </defs>"""


def _corner_brackets(w: int, h: int, pal: Palette, arm: int = 10) -> str:
    return f"""  <path d="M 3 {arm+3} L 3 3 L {arm+3} 3" stroke="{pal.accent}" stroke-width="1.5" fill="none" stroke-linecap="square"/>
  <path d="M {w-arm-3} 3 L {w-3} 3 L {w-3} {arm+3}" stroke="{pal.accent}" stroke-width="1.5" fill="none" stroke-linecap="square"/>
  <path d="M 3 {h-arm-3} L 3 {h-3} L {arm+3} {h-3}" stroke="{pal.accent}" stroke-width="1.5" fill="none" stroke-linecap="square"/>
  <path d="M {w-arm-3} {h-3} L {w-3} {h-3} L {w-3} {h-arm-3}" stroke="{pal.accent}" stroke-width="1.5" fill="none" stroke-linecap="square"/>"""


def _tempest_crest(cx: float, cy: float, pal: Palette, s: int = 14) -> str:
    """Octagon + rotating diamond + center pulse. The Rimuru sigil."""
    edge = round(s * 0.4, 2)
    half = round(s * 0.55, 2)
    return f"""  <g transform="translate({cx} {cy})">
    <polygon points="-{s},-{edge} -{edge},-{s} {edge},-{s} {s},-{edge} {s},{edge} {edge},{s} -{edge},{s} -{s},{edge}" fill="none" stroke="{pal.accent}" stroke-width="1" stroke-dasharray="2 2" opacity="0.65"/>
    <polygon points="0,-{half} {half},0 0,{half} -{half},0" fill="{pal.accent}" opacity="0.35"/>
    <polygon points="0,-{edge} {edge},0 0,{edge} -{edge},0" fill="{pal.glow}"/>
    <circle cx="0" cy="0" r="1.6" fill="{pal.shimmer}"/>
  </g>"""


def _char_px_serif(size: int, spacing: float) -> float:
    return size * 0.72 + spacing


def _char_px_mono(size: int, spacing: float) -> float:
    return size * 0.62 + spacing


def skill_notice(
    title: str,
    subtitle: str,
    notice_id: str,
    rank: str,
    pal: Palette,
) -> str:
    """Section header: anime-style 'Unique Skill' system notice."""
    title_upper = title.upper()
    subtitle_upper = subtitle.upper()
    notice_upper = notice_id.upper()
    rank_upper = rank.upper()

    title_w = len(title_upper) * _char_px_serif(15, 2.4)
    subtitle_w = len(subtitle_upper) * _char_px_mono(9, 1.3)
    notice_w = len(notice_upper) * _char_px_mono(8, 1.2)
    rank_w = len(rank_upper) * _char_px_serif(13, 1.4)

    content_w = max(title_w, subtitle_w)
    right_w = max(notice_w, rank_w) + 10

    crest_area = 54
    pad = 14
    width = int(crest_area + 10 + content_w + 14 + right_w + 16)
    width = max(340, width)
    height = 64

    crest_cx = crest_area / 2 + 4
    crest_cy = height / 2
    left_sep_x = crest_area + 4
    content_x = crest_area + 14
    right_sep_x = width - right_w - 14
    right_text_x = right_sep_x + 6

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        _defs(pal),
        f'  <rect x="0.5" y="0.5" width="{width - 1}" height="{height - 1}" fill="url(#bg)" stroke="{pal.border}"/>',
        f'  <rect x="0.5" y="0.5" width="{width - 1}" height="{height - 1}" fill="url(#grid)"/>',
        _corner_brackets(width, height, pal),
        _tempest_crest(crest_cx, crest_cy, pal, s=15),
        f'  <line x1="{left_sep_x}" y1="14" x2="{left_sep_x}" y2="{height - 14}" stroke="{pal.border}" stroke-width="1" opacity="0.7"/>',
        f'  <text x="{content_x}" y="29" font-family="{SERIF}" font-size="15" font-weight="700" font-style="italic" letter-spacing="2.4" fill="url(#titleGrad)">{title_upper}</text>',
        f'  <text x="{content_x}" y="47" font-family="{MONO}" font-size="9" letter-spacing="1.3" fill="{pal.subtitle}">{subtitle_upper}</text>',
        f'  <line x1="{right_sep_x}" y1="14" x2="{right_sep_x}" y2="{height - 14}" stroke="{pal.border}" stroke-width="1" opacity="0.7"/>',
        f'  <text x="{right_text_x}" y="26" font-family="{MONO}" font-size="8" letter-spacing="1.2" fill="{pal.subtitle}">{notice_upper}</text>',
        f'  <text x="{right_text_x}" y="44" font-family="{SERIF}" font-size="13" font-weight="700" font-style="italic" letter-spacing="1.4" fill="{pal.glow}">{rank_upper}</text>',
        f'  <line x1="14" y1="{height - 4}" x2="{width - 14}" y2="{height - 4}" stroke="url(#shim)" stroke-width="1"/>',
        "</svg>",
        "",
    ]
    return "\n".join(parts)


def stat_card(value: str, label: str, pal: Palette, color_key: str) -> str:
    width = 184
    height = 80
    color = getattr(pal, color_key)

    num_w = len(value) * 17 + 8

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        _defs(pal),
        f'  <rect x="0.5" y="0.5" width="{width - 1}" height="{height - 1}" fill="url(#bg)" stroke="{pal.border}"/>',
        f'  <rect x="0.5" y="0.5" width="{width - 1}" height="{height - 1}" fill="url(#grid)"/>',
        _corner_brackets(width, height, pal),
        _tempest_crest(width - 22, 20, pal, s=8),
        f'  <text x="16" y="24" font-family="{MONO}" font-size="8" letter-spacing="1.6" fill="{pal.subtitle}">SKILL LVL</text>',
        f'  <text x="16" y="54" font-family="{SERIF}" font-style="italic" font-size="28" font-weight="700" fill="{color}">{value}</text>',
        f'  <line x1="16" y1="60" x2="{16 + num_w}" y2="60" stroke="{color}" stroke-width="1.5"/>',
        f'  <text x="16" y="73" font-family="{MONO}" font-size="9" letter-spacing="1.2" fill="{pal.subtitle}">{label.upper()}</text>',
        "</svg>",
        "",
    ]
    return "\n".join(parts)


def skill_tag(text: str, rarity: str, pal: Palette) -> str:
    rarity_colors = {
        "common": pal.common,
        "rare": pal.rare,
        "unique": pal.unique,
        "legendary": pal.gold,
    }
    color = rarity_colors.get(rarity, pal.accent)

    height = 28
    text_w = len(text) * _char_px_mono(10, 1)
    width = int(text_w + 40)
    cy = height / 2

    hex_pts = f"7,0 {width - 7},0 {width},{cy} {width - 7},{height} 7,{height} 0,{cy}"

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        _defs(pal),
        f'  <polygon points="{hex_pts}" fill="url(#bg)" stroke="{color}" stroke-width="1.2"/>',
        f'  <polygon points="16,7 22,{cy} 16,{height - 7} 10,{cy}" fill="{color}"/>',
        f'  <text x="28" y="18" font-family="{MONO}" font-size="10" font-weight="600" letter-spacing="1.2" fill="{pal.title}">{text}</text>',
        "</svg>",
        "",
    ]
    return "\n".join(parts)


def divider(pal: Palette) -> str:
    width = 820
    height = 16
    cy = height // 2
    cx = width // 2

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        "  <defs>",
        '    <linearGradient id="line" x1="0" y1="0" x2="1" y2="0">',
        f'      <stop offset="0" stop-color="{pal.accent}" stop-opacity="0"/>',
        f'      <stop offset="0.5" stop-color="{pal.accent}" stop-opacity="0.7"/>',
        f'      <stop offset="1" stop-color="{pal.accent}" stop-opacity="0"/>',
        "    </linearGradient>",
        "  </defs>",
        f'  <line x1="0" y1="{cy}" x2="{width}" y2="{cy}" stroke="url(#line)" stroke-width="1"/>',
        f'  <circle cx="{cx - 20}" cy="{cy}" r="1" fill="{pal.accent}" opacity="0.5"/>',
        f'  <circle cx="{cx - 12}" cy="{cy}" r="1.5" fill="{pal.accent}" opacity="0.7"/>',
        f'  <circle cx="{cx}" cy="{cy}" r="5" fill="{pal.bg_a}" stroke="{pal.accent}" stroke-width="1.2"/>',
        f'  <circle cx="{cx}" cy="{cy}" r="2.5" fill="{pal.glow}"/>',
        f'  <circle cx="{cx + 12}" cy="{cy}" r="1.5" fill="{pal.accent}" opacity="0.7"/>',
        f'  <circle cx="{cx + 20}" cy="{cy}" r="1" fill="{pal.accent}" opacity="0.5"/>',
        "</svg>",
        "",
    ]
    return "\n".join(parts)


def hero_banner(pal: Palette) -> str:
    """720x240 hero banner: RIMURU title in italic Georgia, Tempest Crest,
    starfield background, corner brackets, version tag."""
    width = 720
    height = 240

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        "  <defs>",
        '    <radialGradient id="sky" cx="0.5" cy="0.4" r="0.8">',
        f'      <stop offset="0" stop-color="{pal.bg_b}"/>',
        f'      <stop offset="1" stop-color="{pal.bg_a}"/>',
        "    </radialGradient>",
        '    <linearGradient id="rim" x1="0" y1="0" x2="1" y2="1">',
        f'      <stop offset="0" stop-color="{pal.glow}"/>',
        f'      <stop offset="0.5" stop-color="{pal.shimmer}"/>',
        f'      <stop offset="1" stop-color="{pal.purple}"/>',
        "    </linearGradient>",
        '    <linearGradient id="shimTop" x1="0" y1="0" x2="1" y2="0">',
        f'      <stop offset="0" stop-color="{pal.accent}" stop-opacity="0"/>',
        f'      <stop offset="0.5" stop-color="{pal.glow}" stop-opacity="0.9"/>',
        f'      <stop offset="1" stop-color="{pal.accent}" stop-opacity="0"/>',
        "    </linearGradient>",
        '    <pattern id="stars" patternUnits="userSpaceOnUse" width="60" height="60">',
        f'      <circle cx="8" cy="12" r="0.8" fill="{pal.shimmer}" opacity="0.4"/>',
        f'      <circle cx="34" cy="28" r="0.5" fill="{pal.glow}" opacity="0.5"/>',
        f'      <circle cx="52" cy="8" r="0.6" fill="{pal.shimmer}" opacity="0.3"/>',
        f'      <circle cx="22" cy="48" r="0.7" fill="{pal.glow}" opacity="0.4"/>',
        f'      <circle cx="46" cy="44" r="0.5" fill="{pal.shimmer}" opacity="0.5"/>',
        "    </pattern>",
        '    <pattern id="grid" patternUnits="userSpaceOnUse" width="12" height="12" patternTransform="rotate(45)">',
        f'      <line x1="0" y1="0" x2="0" y2="12" stroke="{pal.accent}" stroke-width="1.2" opacity="0.04"/>',
        "    </pattern>",
        "  </defs>",
        f'  <rect x="0" y="0" width="{width}" height="{height}" fill="url(#sky)"/>',
        f'  <rect x="0" y="0" width="{width}" height="{height}" fill="url(#stars)"/>',
        f'  <rect x="0" y="0" width="{width}" height="{height}" fill="url(#grid)"/>',
        f'  <rect x="0.5" y="0.5" width="{width - 1}" height="{height - 1}" fill="none" stroke="{pal.border}"/>',
        _corner_brackets(width, height, pal, arm=22),
        _tempest_crest(width / 2, 64, pal, s=26),
        f'  <text x="{width / 2}" y="152" text-anchor="middle" font-family="{SERIF}" font-size="84" font-weight="700" font-style="italic" letter-spacing="10" fill="url(#rim)">RIMURU</text>',
        f'  <text x="{width / 2}" y="182" text-anchor="middle" font-family="{MONO}" font-size="11" letter-spacing="6" fill="{pal.subtitle}">TEMPEST SYSTEM</text>',
        f'  <line x1="{width / 2 - 90}" y1="196" x2="{width / 2 + 90}" y2="196" stroke="url(#shimTop)" stroke-width="1"/>',
        f'  <text x="{width / 2}" y="216" text-anchor="middle" font-family="{MONO}" font-size="10" letter-spacing="2.8" fill="{pal.glow}">UNIQUE SKILL -- COST CONTROL FOR AI CODING AGENTS</text>',
        f'  <text x="30" y="30" font-family="{MONO}" font-size="9" letter-spacing="1.4" fill="{pal.subtitle}">TEMPEST // 001</text>',
        f'  <text x="{width - 30}" y="30" text-anchor="end" font-family="{MONO}" font-size="9" letter-spacing="1.4" fill="{pal.glow}">v0.4.0 // BENIMARU</text>',
        "</svg>",
        "",
    ]
    return "\n".join(parts)


SECTIONS = [
    ("why", "Rimuru", "Cost control for AI coding agents", "NOTICE 000", "PROLOGUE"),
    ("quickstart", "Quick Start", "One line install - Four binaries", "NOTICE 001", "RANK C"),
    ("agents", "Works With Every Agent", "Eight tools - one dashboard", "NOTICE 002", "RANK A"),
    ("budget", "Budget Engine", "Hard caps - four enforcement levels", "SKILL 003", "RANK SS"),
    ("runaway", "Runaway Detection", "Four patterns - severity scoring", "SKILL 004", "RANK S"),
    ("guard", "Guard Wrapper", "Kill agents at the cost limit", "SKILL 005", "RANK S"),
    ("compression", "Output Compression", "Six strategies - auto routing", "SKILL 006", "RANK A"),
    ("interfaces", "Four Interfaces", "CLI - Web UI - TUI - Desktop", "NOTICE 007", "RANK B"),
    ("advisor", "Hardware Advisor", "Run models locally - see savings", "NOTICE 008", "RANK B"),
    ("architecture", "Architecture", "Worker - function - trigger", "NOTICE 009", "RANK S"),
    ("api", "API Reference", "HTTP + iii trigger - same functions", "NOTICE 010", "RANK A"),
    ("development", "Development", "Build - test - ship", "NOTICE 011", "RANK C"),
    ("license", "License", "Apache 2.0 - made in public", "NOTICE 012", "RANK -"),
]

STATS = [
    ("agents", "6", "agents tracked", "glow"),
    ("functions", "60+", "iii functions", "glow"),
    ("interfaces", "4", "interfaces", "purple"),
    ("caps", "4", "budget cap levels", "glow"),
    ("strategies", "6", "compression modes", "purple"),
    ("endpoints", "46", "http endpoints", "glow"),
]

PILLS = [
    ("rust", "RUST 1.85", "unique"),
    ("apache2", "APACHE 2.0", "common"),
    ("v040", "v0.4.0 BENIMARU", "legendary"),
    ("iii", "III-ENGINE", "rare"),
    ("tauri", "TAURI V2", "rare"),
]


def generate(pal: Palette, out_dir: Path) -> int:
    out_dir.mkdir(parents=True, exist_ok=True)
    n = 0
    for key, title, subtitle, notice_id, rank in SECTIONS:
        (out_dir / f"section-{key}.svg").write_text(
            skill_notice(title, subtitle, notice_id, rank, pal)
        )
        n += 1
    for key, value, label, color_key in STATS:
        (out_dir / f"stat-{key}.svg").write_text(stat_card(value, label, pal, color_key))
        n += 1
    for key, text, rarity in PILLS:
        (out_dir / f"pill-{key}.svg").write_text(skill_tag(text, rarity, pal))
        n += 1
    (out_dir / "divider.svg").write_text(divider(pal))
    n += 1
    return n


def main() -> None:
    dark_n = generate(DARK, DARK_DIR)
    light_n = generate(LIGHT, LIGHT_DIR)
    BANNER_PATH.write_text(hero_banner(DARK))
    print(f"wrote {dark_n} dark tags -> {DARK_DIR.relative_to(ROOT)}")
    print(f"wrote {light_n} light tags -> {LIGHT_DIR.relative_to(ROOT)}")
    print(f"wrote hero banner -> {BANNER_PATH.relative_to(ROOT)}")
    print(f"total: {dark_n + light_n + 1} svg files")


if __name__ == "__main__":
    main()
