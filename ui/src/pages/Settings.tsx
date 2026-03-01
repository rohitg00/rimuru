import { useState, useEffect } from "react";
import { useQuery } from "../hooks/useQuery";
import { apiPut } from "../api/client";
import type { AppConfig } from "../api/types";

interface FieldDef {
  key: keyof AppConfig;
  label: string;
  description: string;
  type: "text" | "number" | "boolean" | "select";
  options?: string[];
  min?: number;
  max?: number;
  step?: number;
}

const FIELDS: FieldDef[] = [
  {
    key: "api_port",
    label: "API Port",
    description: "HTTP API port for the server",
    type: "number",
    min: 1024,
    max: 65535,
  },
  {
    key: "theme",
    label: "Theme",
    description: "UI color theme",
    type: "text",
  },
  {
    key: "auto_detect_agents",
    label: "Auto Detect Agents",
    description: "Automatically detect and connect agents on startup",
    type: "boolean",
  },
  {
    key: "auto_sync_models",
    label: "Auto Sync Models",
    description: "Automatically sync model pricing data",
    type: "boolean",
  },
  {
    key: "budget_monthly",
    label: "Monthly Budget ($)",
    description: "Monthly cost budget limit. Set 0 for unlimited",
    type: "number",
    min: 0,
    step: 5,
  },
  {
    key: "budget_alert_threshold",
    label: "Budget Alert (%)",
    description: "Alert when spending reaches this fraction of budget (0-1)",
    type: "number",
    min: 0,
    max: 1,
    step: 0.1,
  },
  {
    key: "log_level",
    label: "Log Level",
    description: "Logging verbosity level",
    type: "select",
    options: ["trace", "debug", "info", "warn", "error"],
  },
  {
    key: "cost_tracking_enabled",
    label: "Cost Tracking",
    description: "Enable cost tracking and analytics",
    type: "boolean",
  },
  {
    key: "enable_hooks",
    label: "Hooks",
    description: "Enable hook event dispatching",
    type: "boolean",
  },
  {
    key: "enable_plugins",
    label: "Plugins",
    description: "Enable plugin system",
    type: "boolean",
  },
  {
    key: "metrics_collection_enabled",
    label: "Metrics Collection",
    description: "Enable system metrics collection",
    type: "boolean",
  },
  {
    key: "session_monitoring_enabled",
    label: "Session Monitoring",
    description: "Enable session monitoring and tracking",
    type: "boolean",
  },
  {
    key: "poll_interval_secs",
    label: "Poll Interval (s)",
    description: "Agent polling interval in seconds",
    type: "number",
    min: 5,
    step: 5,
  },
  {
    key: "metrics_interval_secs",
    label: "Metrics Interval (s)",
    description: "Metrics collection interval in seconds",
    type: "number",
    min: 10,
    step: 10,
  },
  {
    key: "model_sync_interval_hours",
    label: "Model Sync (hrs)",
    description: "Model pricing sync interval in hours",
    type: "number",
    min: 1,
    step: 1,
  },
  {
    key: "currency",
    label: "Currency",
    description: "Currency for cost display",
    type: "text",
  },
];

export default function Settings() {
  const { data: config, refetch } = useQuery<AppConfig>("/config", 30000);
  const [form, setForm] = useState<Partial<AppConfig>>({});
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (config) {
      setForm(config);
    }
  }, [config]);

  function updateField(key: keyof AppConfig, value: unknown) {
    setForm((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  }

  async function handleSave() {
    setSaving(true);
    setError(null);
    try {
      await apiPut("/config", form);
      setSaved(true);
      refetch();
      setTimeout(() => setSaved(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save");
    } finally {
      setSaving(false);
    }
  }

  function handleReset() {
    if (config) {
      setForm(config);
      setSaved(false);
      setError(null);
    }
  }

  const hasChanges = config && JSON.stringify(form) !== JSON.stringify(config);

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h2 className="text-xl font-bold text-[var(--text-primary)]">
          Settings
        </h2>
        <p className="text-sm text-[var(--text-secondary)]">
          Configure the Rimuru server
        </p>
      </div>

      {error && (
        <div className="rounded-lg border border-[var(--error)]/30 bg-[var(--error)]/10 px-4 py-3 text-sm text-[var(--error)]">
          {error}
        </div>
      )}

      {saved && (
        <div className="rounded-lg border border-[var(--success)]/30 bg-[var(--success)]/10 px-4 py-3 text-sm text-[var(--success)]">
          Settings saved successfully
        </div>
      )}

      <div className="space-y-1">
        {FIELDS.map((field) => (
          <div
            key={field.key}
            className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5"
          >
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
              <div className="flex-1">
                <label className="text-sm font-medium text-[var(--text-primary)]">
                  {field.label}
                </label>
                <p className="text-xs text-[var(--text-secondary)] mt-0.5">
                  {field.description}
                </p>
              </div>
              <div className="w-full sm:w-48 shrink-0">
                {field.type === "boolean" ? (
                  <button
                    onClick={() =>
                      updateField(field.key, !(form[field.key] as boolean))
                    }
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                      form[field.key]
                        ? "bg-[var(--accent)]"
                        : "bg-[var(--bg-tertiary)] border border-[var(--border)]"
                    }`}
                  >
                    <span
                      className={`inline-block h-4 w-4 rounded-full bg-white shadow-sm transition-transform ${
                        form[field.key] ? "translate-x-6" : "translate-x-1"
                      }`}
                    />
                  </button>
                ) : field.type === "select" ? (
                  <select
                    value={String(form[field.key] ?? "")}
                    onChange={(e) => updateField(field.key, e.target.value)}
                    className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border)] text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
                  >
                    {field.options?.map((opt) => (
                      <option key={opt} value={opt}>
                        {opt}
                      </option>
                    ))}
                  </select>
                ) : field.type === "number" ? (
                  <input
                    type="number"
                    value={
                      form[field.key] != null ? Number(form[field.key]) : ""
                    }
                    onChange={(e) =>
                      updateField(field.key, parseFloat(e.target.value) || 0)
                    }
                    min={field.min}
                    max={field.max}
                    step={field.step}
                    className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border)] text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] font-mono"
                  />
                ) : (
                  <input
                    type="text"
                    value={String(form[field.key] ?? "")}
                    onChange={(e) => updateField(field.key, e.target.value)}
                    className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border)] text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
                  />
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="flex items-center justify-between pt-4 border-t border-[var(--border)]">
        <button
          onClick={handleReset}
          disabled={!hasChanges}
          className="px-4 py-2 text-sm rounded-lg bg-[var(--bg-secondary)] text-[var(--text-secondary)] border border-[var(--border)] hover:text-[var(--text-primary)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Reset
        </button>
        <button
          onClick={handleSave}
          disabled={!hasChanges || saving}
          className="px-6 py-2 text-sm font-medium rounded-lg bg-[var(--accent)] text-white hover:opacity-90 disabled:opacity-40 disabled:cursor-not-allowed transition-opacity"
        >
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>
    </div>
  );
}
