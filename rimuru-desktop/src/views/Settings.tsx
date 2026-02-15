import { useTheme, ThemeName } from "@/hooks/useTheme";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands } from "@/lib/tauri";
import {
  SettingsSection,
  SettingRow,
  Select,
  Input,
  InfoRow,
} from "@/components/SettingsForm";
import styles from "./Settings.module.css";

const SYNC_INTERVAL_OPTIONS = [
  { value: "15", label: "15 minutes" },
  { value: "30", label: "30 minutes" },
  { value: "60", label: "1 hour" },
  { value: "360", label: "6 hours" },
  { value: "1440", label: "24 hours" },
  { value: "manual", label: "Manual only" },
];

const DEFAULT_MODEL_OPTIONS = [
  { value: "gpt-4o", label: "GPT-4o" },
  { value: "gpt-4o-mini", label: "GPT-4o Mini" },
  { value: "claude-3-5-sonnet", label: "Claude 3.5 Sonnet" },
  { value: "claude-3-opus", label: "Claude 3 Opus" },
  { value: "gemini-pro", label: "Gemini Pro" },
];

export default function Settings() {
  const { theme, setTheme, themes } = useTheme();
  const queryClient = useQueryClient();

  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: commands.getSettings,
  });

  const { data: dbStats } = useQuery({
    queryKey: ["db-stats"],
    queryFn: commands.getDbStats,
  });

  const { data: syncStatus } = useQuery({
    queryKey: ["sync", "status"],
    queryFn: commands.getSyncStatus,
  });

  const saveMutation = useMutation({
    mutationFn: (updated: { sync_interval: string; default_model: string; session_timeout: string }) =>
      commands.saveSettings(updated),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });

  const syncMutation = useMutation({
    mutationFn: commands.triggerSync,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sync", "status"] });
      queryClient.invalidateQueries({ queryKey: ["costs"] });
    },
  });

  const updateSetting = (key: string, value: string) => {
    if (!settings) return;
    saveMutation.mutate({ ...settings, [key]: value });
  };

  const themeLabels: Record<string, string> = {
    rimuru: "Rimuru Slime",
    "tokyo-night": "Tokyo Night",
    catppuccin: "Catppuccin",
    dracula: "Dracula",
    nord: "Nord",
    light: "Light",
    monokai: "Monokai",
    "gruvbox-dark": "Gruvbox Dark",
    "github-light": "GitHub Light",
    solarized: "Solarized",
    "one-light": "One Light",
    "gruvbox-light": "Gruvbox Light",
    "ayu-light": "Ayu Light",
    "catppuccin-latte": "Catppuccin Latte",
    pedurple: "Pedurple",
  };
  const themeOptions = themes.map((t) => ({
    value: t,
    label: themeLabels[t] || t,
  }));

  return (
    <div className={styles.container}>
      <h1 className={styles.title}>Settings</h1>

      <SettingsSection title="Appearance">
        <SettingRow
          label="Theme"
          description="Choose your preferred color theme"
        >
          <Select
            value={theme}
            onChange={(v) => setTheme(v as ThemeName)}
            options={themeOptions}
          />
        </SettingRow>
      </SettingsSection>

      <SettingsSection title="Model Pricing Sync">
        <SettingRow
          label="Sync Interval"
          description="How often to automatically sync model pricing"
        >
          <Select
            value={settings?.sync_interval ?? "60"}
            onChange={(v) => updateSetting("sync_interval", v)}
            options={SYNC_INTERVAL_OPTIONS}
          />
        </SettingRow>

        <SettingRow
          label="Last Sync"
          description={
            syncStatus?.last_sync
              ? new Date(syncStatus.last_sync).toLocaleString()
              : "Never synced"
          }
        >
          <button
            className="btn btn-primary"
            onClick={() => syncMutation.mutate()}
            disabled={syncMutation.isPending}
          >
            {syncMutation.isPending ? "Syncing..." : "Sync Now"}
          </button>
        </SettingRow>

        {syncStatus?.provider_statuses && syncStatus.provider_statuses.length > 0 && (
          <div className={styles.providerList}>
            {syncStatus.provider_statuses.map((provider) => (
              <div key={provider.name} className={styles.provider}>
                <span className={styles.providerName}>{provider.name}</span>
                <span className={styles.providerModels}>
                  {provider.model_count} models
                </span>
                <span
                  className={`badge ${provider.enabled ? "badge-success" : "badge-warning"}`}
                >
                  {provider.enabled ? "Enabled" : "Disabled"}
                </span>
              </div>
            ))}
          </div>
        )}
      </SettingsSection>

      <SettingsSection title="Agent Defaults">
        <SettingRow
          label="Default Model"
          description="Default model for new agent sessions"
        >
          <Select
            value={settings?.default_model ?? "gpt-4o"}
            onChange={(v) => updateSetting("default_model", v)}
            options={DEFAULT_MODEL_OPTIONS}
          />
        </SettingRow>

        <SettingRow
          label="Session Timeout"
          description="Minutes of inactivity before session ends"
        >
          <Input
            type="number"
            value={settings?.session_timeout ?? "30"}
            onChange={(v) => updateSetting("session_timeout", v)}
            placeholder="30"
          />
        </SettingRow>
      </SettingsSection>

      <SettingsSection title="Database">
        <SettingRow
          label="Database Path"
          description="Location of the SQLite database file"
        >
          <Input
            value={dbStats?.db_path ?? "~/.rimuru/rimuru.db"}
            onChange={() => {}}
            placeholder="~/.rimuru/rimuru.db"
            disabled
          />
        </SettingRow>

        <InfoRow label="Database Size" value={dbStats?.db_size_display ?? "..."} />
        <InfoRow label="Total Sessions" value={dbStats?.total_sessions?.toLocaleString() ?? "..."} />
        <InfoRow label="Total Agents" value={dbStats?.total_agents?.toLocaleString() ?? "..."} />
      </SettingsSection>

      <SettingsSection title="About">
        <InfoRow label="Version" value="0.1.0" />
        <InfoRow label="License" value="Apache-2.0" />
        <InfoRow
          label="Repository"
          value={
            <a
              href="https://github.com/rohitg00/rimuru"
              target="_blank"
              rel="noopener noreferrer"
            >
              github.com/rohitg00/rimuru
            </a>
          }
        />
        <InfoRow label="Build" value={`${new Date().getFullYear()}.1.0`} />
      </SettingsSection>
    </div>
  );
}
