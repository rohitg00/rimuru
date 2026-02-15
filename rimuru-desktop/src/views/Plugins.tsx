import { useState } from "react";
import {
  Puzzle,
  Package,
  Store,
  RefreshCw,
  Search,
  Power,
  PowerOff,
  Trash2,
  Settings,
  Download,
  ExternalLink,
  AlertCircle,
  CheckCircle,
  XCircle,
  FolderOpen,
} from "lucide-react";
import { usePlugins, useEnablePlugin, useDisablePlugin, useUninstallPlugin, useInstallPlugin } from "@/hooks/usePlugins";
import { Plugin } from "@/lib/tauri";
import { Spinner } from "@/components/Spinner/Spinner";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import PluginConfig from "@/components/PluginConfig";
import styles from "./Plugins.module.css";
import clsx from "clsx";

type Tab = "installed" | "builtin" | "available";

const CAPABILITY_OPTIONS = [
  { value: "all", label: "All Capabilities" },
  { value: "agent", label: "Agent" },
  { value: "exporter", label: "Exporter" },
  { value: "notifier", label: "Notifier" },
  { value: "view", label: "View" },
  { value: "hook", label: "Hook" },
];

export default function Plugins() {
  const [activeTab, setActiveTab] = useState<Tab>("installed");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCapability, setSelectedCapability] = useState("all");
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);
  const [showInstallModal, setShowInstallModal] = useState(false);
  const [installPath, setInstallPath] = useState("");

  const { data: plugins, isLoading, refetch } = usePlugins(
    activeTab === "available",
    selectedCapability === "all" ? undefined : selectedCapability
  );

  const enableMutation = useEnablePlugin();
  const disableMutation = useDisablePlugin();
  const uninstallMutation = useUninstallPlugin();
  const installMutation = useInstallPlugin();

  const filteredPlugins = plugins?.filter((plugin) => {
    if (activeTab === "installed" && plugin.is_builtin) return false;
    if (activeTab === "builtin" && !plugin.is_builtin) return false;
    if (activeTab === "available" && plugin.status !== "available") return false;

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return (
        plugin.name.toLowerCase().includes(query) ||
        plugin.description.toLowerCase().includes(query) ||
        plugin.author.toLowerCase().includes(query)
      );
    }
    return true;
  });

  const installedCount = plugins?.filter((p) => !p.is_builtin && p.status !== "available").length ?? 0;
  const builtinCount = plugins?.filter((p) => p.is_builtin).length ?? 0;
  const enabledCount = plugins?.filter((p) => p.enabled).length ?? 0;
  const errorCount = plugins?.filter((p) => p.error).length ?? 0;

  const handleToggleEnabled = async (plugin: Plugin) => {
    if (plugin.enabled) {
      await disableMutation.mutateAsync(plugin.id);
    } else {
      await enableMutation.mutateAsync(plugin.id);
    }
    refetch();
  };

  const handleUninstall = async (plugin: Plugin) => {
    if (plugin.is_builtin) return;
    if (confirm(`Are you sure you want to uninstall "${plugin.name}"?`)) {
      try {
        await uninstallMutation.mutateAsync({ pluginId: plugin.id });
        refetch();
      } catch (error) {
        alert(error);
      }
    }
  };

  const handleInstall = async () => {
    if (!installPath) return;
    try {
      await installMutation.mutateAsync({ source: installPath, auto_enable: true });
      setShowInstallModal(false);
      setInstallPath("");
      refetch();
    } catch (error) {
      alert(error);
    }
  };

  const getStatusIcon = (plugin: Plugin) => {
    if (plugin.error) return <XCircle size={16} className={styles.statusError} />;
    if (plugin.enabled) return <CheckCircle size={16} className={styles.statusEnabled} />;
    return <AlertCircle size={16} className={styles.statusDisabled} />;
  };

  const getCapabilityBadge = (capability: string) => {
    const colors: Record<string, string> = {
      agent: "badge-agent",
      exporter: "badge-exporter",
      notifier: "badge-notifier",
      view: "badge-view",
      hook: "badge-hook",
      custom: "badge-custom",
    };
    return colors[capability] || "badge-custom";
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Plugins</h1>
        <div className={styles.headerActions}>
          <button
            className="btn btn-primary"
            onClick={() => setShowInstallModal(true)}
          >
            <Download size={16} />
            Install Plugin
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => refetch()}
            disabled={isLoading}
          >
            <RefreshCw size={16} className={isLoading ? styles.spinning : ""} />
            Refresh
          </button>
        </div>
      </div>

      <div className={styles.stats}>
        <div className={styles.statCard}>
          <span className={styles.statValue}>{installedCount}</span>
          <span className={styles.statLabel}>Installed</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statValue}>{enabledCount}</span>
          <span className={styles.statLabel}>Enabled</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statValue}>{builtinCount}</span>
          <span className={styles.statLabel}>Built-in</span>
        </div>
        <div className={styles.statCard}>
          <span className={clsx(styles.statValue, errorCount > 0 && styles.statError)}>
            {errorCount}
          </span>
          <span className={styles.statLabel}>Errors</span>
        </div>
      </div>

      <div className={styles.tabs}>
        <button
          className={clsx(styles.tab, activeTab === "installed" && styles.active)}
          onClick={() => setActiveTab("installed")}
        >
          <Package size={16} />
          Installed
          <span className={styles.tabCount}>{installedCount}</span>
        </button>
        <button
          className={clsx(styles.tab, activeTab === "builtin" && styles.active)}
          onClick={() => setActiveTab("builtin")}
        >
          <Puzzle size={16} />
          Built-in
          <span className={styles.tabCount}>{builtinCount}</span>
        </button>
        <button
          className={clsx(styles.tab, activeTab === "available" && styles.active)}
          onClick={() => setActiveTab("available")}
        >
          <Store size={16} />
          Available
        </button>
      </div>

      <div className={styles.filters}>
        <div className={styles.searchWrapper}>
          <Search size={16} className={styles.searchIcon} />
          <input
            type="text"
            placeholder="Search plugins..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className={styles.searchInput}
          />
        </div>
        <select
          value={selectedCapability}
          onChange={(e) => setSelectedCapability(e.target.value)}
          className={styles.filterSelect}
        >
          {CAPABILITY_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div className={styles.content}>
        {isLoading ? (
          <div className={styles.loading}>
            <Spinner />
            <span>Loading plugins...</span>
          </div>
        ) : filteredPlugins && filteredPlugins.length > 0 ? (
          <div className={styles.pluginList}>
            {filteredPlugins.map((plugin) => (
              <div
                key={plugin.id}
                className={clsx(
                  styles.pluginCard,
                  selectedPlugin?.id === plugin.id && styles.selected
                )}
              >
                <div className={styles.pluginHeader}>
                  <div className={styles.pluginInfo}>
                    {getStatusIcon(plugin)}
                    <div className={styles.pluginMeta}>
                      <h3 className={styles.pluginName}>{plugin.name}</h3>
                      <span className={styles.pluginVersion}>v{plugin.version}</span>
                    </div>
                  </div>
                  <div className={styles.pluginActions}>
                    {plugin.status !== "available" && (
                      <>
                        <button
                          className={clsx(styles.actionBtn, plugin.enabled && styles.enabled)}
                          onClick={() => handleToggleEnabled(plugin)}
                          title={plugin.enabled ? "Disable" : "Enable"}
                        >
                          {plugin.enabled ? <Power size={16} /> : <PowerOff size={16} />}
                        </button>
                        <button
                          className={styles.actionBtn}
                          onClick={() => setSelectedPlugin(plugin)}
                          title="Configure"
                        >
                          <Settings size={16} />
                        </button>
                        {!plugin.is_builtin && (
                          <button
                            className={clsx(styles.actionBtn, styles.danger)}
                            onClick={() => handleUninstall(plugin)}
                            title="Uninstall"
                          >
                            <Trash2 size={16} />
                          </button>
                        )}
                      </>
                    )}
                    {plugin.status === "available" && (
                      <button
                        className="btn btn-primary btn-sm"
                        onClick={() => {
                          if (plugin.repository) {
                            window.open(plugin.repository, "_blank");
                          }
                        }}
                      >
                        <Download size={14} />
                        Get
                      </button>
                    )}
                  </div>
                </div>

                <p className={styles.pluginDescription}>{plugin.description}</p>

                <div className={styles.pluginFooter}>
                  <div className={styles.capabilities}>
                    {plugin.capabilities.map((cap) => (
                      <span
                        key={cap}
                        className={clsx(styles.capabilityBadge, styles[getCapabilityBadge(cap)])}
                      >
                        {cap}
                      </span>
                    ))}
                  </div>
                  <div className={styles.pluginAuthor}>
                    by {plugin.author}
                    {plugin.repository && (
                      <a
                        href={plugin.repository}
                        target="_blank"
                        rel="noopener noreferrer"
                        className={styles.repoLink}
                      >
                        <ExternalLink size={12} />
                      </a>
                    )}
                  </div>
                </div>

                {plugin.error && (
                  <div className={styles.pluginError}>
                    <AlertCircle size={14} />
                    {plugin.error}
                  </div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={Puzzle}
            title="No plugins"
            description={
              activeTab === "installed"
                ? "Install plugins from the Available tab"
                : activeTab === "builtin"
                ? "All built-in plugins are shown"
                : "No plugins available in the marketplace"
            }
          />
        )}
      </div>

      {selectedPlugin && (
        <PluginConfig
          plugin={selectedPlugin}
          onClose={() => setSelectedPlugin(null)}
        />
      )}

      {showInstallModal && (
        <div className={styles.modalOverlay} onClick={() => setShowInstallModal(false)}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <h2 className={styles.modalTitle}>Install Plugin</h2>
            <p className={styles.modalDescription}>
              Enter the path to a local plugin directory or manifest file.
            </p>
            <div className={styles.inputGroup}>
              <FolderOpen size={16} className={styles.inputIcon} />
              <input
                type="text"
                placeholder="/path/to/plugin"
                value={installPath}
                onChange={(e) => setInstallPath(e.target.value)}
                className={styles.modalInput}
              />
            </div>
            <div className={styles.modalActions}>
              <button
                className="btn btn-secondary"
                onClick={() => setShowInstallModal(false)}
              >
                Cancel
              </button>
              <button
                className="btn btn-primary"
                onClick={handleInstall}
                disabled={!installPath || installMutation.isPending}
              >
                {installMutation.isPending ? "Installing..." : "Install"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
