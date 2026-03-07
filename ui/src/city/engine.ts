import type { Agent, StreamEvent } from "../api/types";
import type { CityCharacter } from "./characters";
import {
  createCharacter,
  updateCharacter,
  updateCharacterStatus,
} from "./characters";
import type { Camera } from "./renderer";
import {
  renderTiles,
  renderCharacter,
  renderLabel,
  renderDistrictLabels,
  renderShadow,
  renderBubble,
} from "./renderer";
import { TILE_SIZE, CHAR_W, CHAR_H } from "./sprites";
import { MAP_W, MAP_H } from "./tilemap";

const MAX_DT = 0.1;

export class CityEngine {
  characters = new Map<string, CityCharacter>();
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  camera: Camera = { x: 0, y: 0, zoom: 3 };
  animationId = 0;
  lastTime = 0;
  running = false;

  dpr = 1;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d")!;
    this.ctx.imageSmoothingEnabled = false;
    this.dpr = window.devicePixelRatio || 1;
    this.resetCamera();
  }

  start(): void {
    if (this.running) return;
    this.running = true;
    this.lastTime = performance.now();
    const loop = (time: number) => {
      if (!this.running) return;
      const dt = Math.min((time - this.lastTime) / 1000, MAX_DT);
      this.lastTime = time;
      this.update(dt);
      this.render();
      this.animationId = requestAnimationFrame(loop);
    };
    this.animationId = requestAnimationFrame(loop);
  }

  stop(): void {
    this.running = false;
    cancelAnimationFrame(this.animationId);
  }

  syncAgents(agents: Agent[]): void {
    const agentIds = new Set(agents.map((a) => a.id));
    for (const agent of agents) {
      const existing = this.characters.get(agent.id);
      if (!existing) {
        this.characters.set(agent.id, createCharacter(agent));
      } else if (existing.status !== agent.status) {
        updateCharacterStatus(existing, agent.status);
      }
    }
    for (const [id, char] of this.characters) {
      if (!agentIds.has(id) && !char.fading) {
        char.fading = true;
      }
    }
  }

  handleStreamEvent(event: StreamEvent): void {
    const data = event.data as Record<string, unknown> | undefined;
    if (!data) return;
    switch (event.type) {
      case "agent_connected": {
        const agent = data as unknown as Agent;
        if (agent.id && !this.characters.has(agent.id)) {
          this.characters.set(agent.id, createCharacter(agent));
        }
        break;
      }
      case "agent_disconnected": {
        const agentId = (data.agent_id ?? data.id) as string;
        const char = this.characters.get(agentId);
        if (char) updateCharacterStatus(char, "disconnected");
        break;
      }
      case "agent_status_changed": {
        const agentId = (data.agent_id ?? data.id) as string;
        const newStatus = data.status as string;
        const char = this.characters.get(agentId);
        if (char && newStatus) updateCharacterStatus(char, newStatus);
        break;
      }
    }
  }

  update(dt: number): void {
    const toRemove: string[] = [];
    for (const [id, char] of this.characters) {
      updateCharacter(char, dt);
      if (char.opacity <= 0) toRemove.push(id);
    }
    for (const id of toRemove) this.characters.delete(id);
  }

  render(): void {
    const { canvas, ctx, camera } = this;
    const w = canvas.width;
    const h = canvas.height;

    ctx.clearRect(0, 0, w, h);
    ctx.imageSmoothingEnabled = false;

    renderTiles(ctx, camera, w, h);

    const chars = [...this.characters.values()];
    const sorted = chars.sort((a, b) => a.y + CHAR_H - (b.y + CHAR_H));

    for (const char of sorted) renderShadow(ctx, char, camera);
    for (const char of sorted) renderCharacter(ctx, char, camera);
    for (const char of sorted) renderBubble(ctx, char, camera);
    for (const char of sorted) renderLabel(ctx, char, camera);

    renderDistrictLabels(ctx, camera, w, h);
  }

  panBy(dx: number, dy: number): void {
    this.camera.x += dx / this.camera.zoom;
    this.camera.y += dy / this.camera.zoom;
    this.clampCamera();
  }

  zoomTo(level: number): void {
    const cx = this.camera.x + this.canvas.width / (2 * this.camera.zoom);
    const cy = this.camera.y + this.canvas.height / (2 * this.camera.zoom);
    this.camera.zoom = Math.max(this.dpr, Math.min(8 * this.dpr, level));
    this.camera.x = cx - this.canvas.width / (2 * this.camera.zoom);
    this.camera.y = cy - this.canvas.height / (2 * this.camera.zoom);
    this.clampCamera();
  }

  zoomBy(delta: number, pivotX: number, pivotY: number): void {
    const worldX = this.camera.x + pivotX / this.camera.zoom;
    const worldY = this.camera.y + pivotY / this.camera.zoom;
    this.camera.zoom = Math.max(
      this.dpr,
      Math.min(8 * this.dpr, this.camera.zoom + delta),
    );
    this.camera.x = worldX - pivotX / this.camera.zoom;
    this.camera.y = worldY - pivotY / this.camera.zoom;
    this.clampCamera();
  }

  centerOnCharacter(agentId: string): void {
    const char = this.characters.get(agentId);
    if (!char) return;
    this.camera.x =
      char.x + CHAR_W / 2 - this.canvas.width / (2 * this.camera.zoom);
    this.camera.y =
      char.y + CHAR_H / 2 - this.canvas.height / (2 * this.camera.zoom);
    this.clampCamera();
  }

  findCharacterAt(canvasX: number, canvasY: number): CityCharacter | null {
    const worldX = this.camera.x + canvasX / this.camera.zoom;
    const worldY = this.camera.y + canvasY / this.camera.zoom;
    for (const char of this.characters.values()) {
      if (
        worldX >= char.x &&
        worldX <= char.x + CHAR_W &&
        worldY >= char.y - 8 &&
        worldY <= char.y + CHAR_H
      ) {
        return char;
      }
    }
    return null;
  }

  resetCamera(): void {
    const mapPxW = MAP_W * TILE_SIZE;
    this.camera.zoom = 3 * this.dpr;
    this.camera.x = mapPxW / 2 - this.canvas.width / (2 * this.camera.zoom);
    this.camera.y =
      12 * TILE_SIZE - this.canvas.height / (2 * this.camera.zoom);
    this.clampCamera();
  }

  private clampCamera(): void {
    const maxX = MAP_W * TILE_SIZE - this.canvas.width / this.camera.zoom;
    const maxY = MAP_H * TILE_SIZE - this.canvas.height / this.camera.zoom;
    this.camera.x = Math.max(0, Math.min(maxX, this.camera.x));
    this.camera.y = Math.max(0, Math.min(maxY, this.camera.y));
  }
}
