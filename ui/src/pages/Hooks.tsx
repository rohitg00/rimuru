import { useState } from "react";
import { useQuery } from "../hooks/useQuery";
import type { HookConfig } from "../api/types";
import StatusBadge from "../components/StatusBadge";

export default function Hooks() {
  const { data: hooks } = useQuery<HookConfig[]>("/hooks", 5000);
  const [selectedHook, setSelectedHook] = useState<string | null>(null);
  const [expandedEvents, setExpandedEvents] = useState<Set<string>>(new Set());

  const hooksByEvent = new Map<string, HookConfig[]>();
  for (const h of hooks ?? []) {
    const list = hooksByEvent.get(h.event) ?? [];
    list.push(h);
    hooksByEvent.set(h.event, list);
  }

  const toggleEvent = (event: string) => {
    setExpandedEvents((prev) => {
      const next = new Set(prev);
      if (next.has(event)) next.delete(event);
      else next.add(event);
      return next;
    });
  };

  const pluginIds = new Set(
    (hooks ?? []).map((h) => h.plugin_id).filter(Boolean),
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-[var(--text-primary)]">
            Hooks
          </h2>
          <p className="text-sm text-[var(--text-secondary)]">
            {hooks?.length ?? 0} hooks from {pluginIds.size} plugin
            {pluginIds.size !== 1 ? "s" : ""}, {hooksByEvent.size} event types
          </p>
        </div>
      </div>

      <p className="text-xs text-[var(--text-secondary)]">
        Discovered from plugin hook configs. Hooks run automatically during
        Claude Code sessions.
      </p>

      <div className="space-y-4">
        {Array.from(hooksByEvent.entries())
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([event, eventHooks]) => {
            const isExpanded = expandedEvents.has(event);
            return (
              <div key={event}>
                <button
                  onClick={() => toggleEvent(event)}
                  className="w-full text-left"
                >
                  <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-2 flex items-center gap-2">
                    <span
                      className={`w-2 h-2 rounded-full transition-colors ${
                        isExpanded
                          ? "bg-[var(--accent)]"
                          : "bg-[var(--text-secondary)]"
                      }`}
                    />
                    {event}
                    <span className="text-xs text-[var(--text-secondary)] font-normal">
                      ({eventHooks.length})
                    </span>
                    <svg
                      className={`w-3 h-3 text-[var(--text-secondary)] transition-transform ml-auto ${
                        isExpanded ? "rotate-180" : ""
                      }`}
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M19 9l-7 7-7-7"
                      />
                    </svg>
                  </h3>
                </button>

                {isExpanded && (
                  <div className="space-y-2 ml-4">
                    {eventHooks.map((hook) => (
                      <div
                        key={hook.id}
                        className={`rounded-xl border bg-[var(--bg-secondary)] p-4 transition-all ${
                          selectedHook === hook.id
                            ? "border-[var(--accent)]"
                            : "border-[var(--border)] hover:border-[var(--accent)]/30"
                        }`}
                      >
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center gap-3">
                            <button
                              onClick={() =>
                                setSelectedHook(
                                  selectedHook === hook.id ? null : hook.id,
                                )
                              }
                              className="font-medium text-sm text-[var(--text-primary)] hover:text-[var(--accent)] transition-colors text-left"
                            >
                              {hook.name}
                            </button>
                            <StatusBadge
                              status={hook.enabled ? "enabled" : "disabled"}
                              size="sm"
                            />
                          </div>
                        </div>

                        <div className="flex items-center gap-4 text-xs text-[var(--text-secondary)]">
                          {hook.plugin_id && (
                            <span className="px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)]">
                              {hook.plugin_id.split("@")[0]}
                            </span>
                          )}
                          {hook.matcher && (
                            <span className="font-mono">
                              match: {hook.matcher}
                            </span>
                          )}
                          <span>timeout: {hook.timeout_ms}ms</span>
                        </div>

                        {selectedHook === hook.id && (
                          <div className="mt-3 pt-3 border-t border-[var(--border)]">
                            <p className="text-[10px] uppercase tracking-wider text-[var(--text-secondary)] mb-1">
                              Script
                            </p>
                            <pre className="text-xs text-[var(--text-primary)] bg-[var(--bg-tertiary)] rounded-lg p-3 overflow-x-auto font-mono whitespace-pre-wrap break-all">
                              {hook.script}
                            </pre>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            );
          })}

        {(hooks ?? []).length === 0 && (
          <div className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-16 text-center space-y-3">
            <p className="text-4xl">&#x26A1;</p>
            <p className="text-[var(--text-primary)] font-medium">
              No hooks configured
            </p>
            <p className="text-sm text-[var(--text-secondary)] max-w-md mx-auto">
              Hooks run scripts in response to Claude Code events. Install
              plugins with hooks to get started.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
