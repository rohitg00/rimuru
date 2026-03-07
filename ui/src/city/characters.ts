import type { Agent } from "../api/types";
import type { CharacterType } from "./sprites";
import { TILE_SIZE, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP } from "./sprites";
import { DISTRICTS, WALKABLE, CITY_MAP, MAP_W, MAP_H } from "./tilemap";
import { findPath } from "./pathfinding";

export type CharState = "IDLE" | "WALK" | "WORK";

export interface CityCharacter {
  agentId: string;
  name: string;
  characterType: CharacterType;
  state: CharState;
  x: number;
  y: number;
  targetX: number;
  targetY: number;
  path: { x: number; y: number }[];
  direction: 0 | 1 | 2 | 3;
  frame: number;
  frameTimer: number;
  idleTimer: number;
  status: string;
  opacity: number;
  fading: boolean;
}

const AGENT_TYPE_MAP: Record<string, CharacterType> = {
  claude_code: "rimuru",
  cursor: "shion",
  codex: "benimaru",
  gemini_cli: "shuna",
  opencode: "souei",
};

const WALK_SPEED = 48;
const FRAME_DURATION = 0.15;
const IDLE_MIN = 1.5;
const IDLE_MAX = 4.0;
const FADE_SPEED = 2.0;

export function mapAgentType(agentType: string): CharacterType {
  return AGENT_TYPE_MAP[agentType] ?? "gabiru";
}

function randomSpawnPoint(): { x: number; y: number } {
  const central = DISTRICTS.filter(
    (d) =>
      d.name === "Central Plaza" ||
      d.name === "Great Hall" ||
      d.name === "Scholar's Quarter",
  );
  const pool = central.length > 0 ? central : DISTRICTS;
  const district = pool[Math.floor(Math.random() * pool.length)];
  const sp =
    district.spawnPoints[
      Math.floor(Math.random() * district.spawnPoints.length)
    ];
  return { x: sp.x * TILE_SIZE, y: sp.y * TILE_SIZE };
}

function randomWalkableTarget(): { x: number; y: number } {
  const minX = 10;
  const maxX = MAP_W - 10;
  const minY = 4;
  const maxY = MAP_H - 10;
  for (let attempt = 0; attempt < 50; attempt++) {
    const tx = minX + Math.floor(Math.random() * (maxX - minX));
    const ty = minY + Math.floor(Math.random() * (maxY - minY));
    if (WALKABLE.has(CITY_MAP[ty][tx])) {
      return { x: tx, y: ty };
    }
  }
  return { x: 19, y: 12 };
}

function districtTarget(districtName: string): { x: number; y: number } {
  const d = DISTRICTS.find((dd) => dd.name === districtName);
  if (!d) return randomWalkableTarget();
  const sp = d.spawnPoints[Math.floor(Math.random() * d.spawnPoints.length)];
  return { x: sp.x, y: sp.y };
}

function randomIdleTime(): number {
  return IDLE_MIN + Math.random() * (IDLE_MAX - IDLE_MIN);
}

export function createCharacter(agent: Agent): CityCharacter {
  const spawn = randomSpawnPoint();
  return {
    agentId: agent.id,
    name: agent.name,
    characterType: mapAgentType(agent.agent_type),
    state: "IDLE",
    x: spawn.x,
    y: spawn.y,
    targetX: spawn.x,
    targetY: spawn.y,
    path: [],
    direction: DIR_DOWN,
    frame: 0,
    frameTimer: 0,
    idleTimer: 0,
    status: agent.status,
    opacity: 0,
    fading: false,
  };
}

function pickTargetForStatus(status: string): { x: number; y: number } {
  switch (status) {
    case "busy":
      return districtTarget("Great Hall");
    case "idle":
      return districtTarget("Scholar's Quarter");
    default:
      return randomWalkableTarget();
  }
}

export function updateCharacter(char: CityCharacter, dt: number): void {
  if (char.opacity < 1 && !char.fading) {
    char.opacity = Math.min(1, char.opacity + FADE_SPEED * dt);
  }

  if (char.fading) {
    char.opacity = Math.max(0, char.opacity - FADE_SPEED * dt);
    return;
  }

  switch (char.state) {
    case "IDLE": {
      char.idleTimer += dt;
      if (char.idleTimer >= randomIdleTime()) {
        char.idleTimer = 0;
        const target = pickTargetForStatus(char.status);
        const startTile = {
          x: Math.floor(char.x / TILE_SIZE),
          y: Math.floor(char.y / TILE_SIZE),
        };
        const path = findPath(CITY_MAP, WALKABLE, startTile, target);
        if (path && path.length > 0) {
          char.path = path;
          char.state = "WALK";
          char.frame = 0;
          char.frameTimer = 0;
        }
      }
      break;
    }
    case "WALK": {
      if (char.path.length === 0) {
        char.state =
          char.status === "idle" || char.status === "busy" ? "WORK" : "IDLE";
        char.idleTimer = 0;
        char.frame = 0;
        break;
      }

      const next = char.path[0];
      const targetPx = next.x * TILE_SIZE;
      const targetPy = next.y * TILE_SIZE;
      const dx = targetPx - char.x;
      const dy = targetPy - char.y;
      const dist = Math.sqrt(dx * dx + dy * dy);

      const step = WALK_SPEED * dt;
      if (dist <= step) {
        char.x = targetPx;
        char.y = targetPy;
        char.path.shift();
      } else {
        char.x += (dx / dist) * step;
        char.y += (dy / dist) * step;
      }

      if (Math.abs(dx) > Math.abs(dy)) {
        char.direction = dx < 0 ? DIR_LEFT : DIR_RIGHT;
      } else {
        char.direction = dy < 0 ? DIR_UP : DIR_DOWN;
      }

      char.frameTimer += dt;
      if (char.frameTimer >= FRAME_DURATION) {
        char.frameTimer -= FRAME_DURATION;
        char.frame = (char.frame + 1) % 4;
      }
      break;
    }
    case "WORK": {
      char.frameTimer += dt;
      char.idleTimer += dt;
      if (char.idleTimer >= IDLE_MAX * 2) {
        char.state = "IDLE";
        char.idleTimer = 0;
        char.frame = 0;
      }
      break;
    }
  }
}

export function updateCharacterStatus(
  char: CityCharacter,
  newStatus: string,
): void {
  char.status = newStatus;
  if (newStatus === "disconnected") {
    char.fading = true;
    return;
  }
  char.fading = false;
  if (newStatus === "error") {
    char.state = "IDLE";
    char.path = [];
    return;
  }
  const target = pickTargetForStatus(newStatus);
  const startTile = {
    x: Math.floor(char.x / TILE_SIZE),
    y: Math.floor(char.y / TILE_SIZE),
  };
  const path = findPath(CITY_MAP, WALKABLE, startTile, target);
  if (path && path.length > 0) {
    char.path = path;
    char.state = "WALK";
    char.frame = 0;
    char.frameTimer = 0;
  }
}
