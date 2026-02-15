import { useState } from "react";
import {
  Webhook,
  ListTree,
  History,
  RefreshCw,
  Search,
  Play,
  Power,
  PowerOff,
  ChevronRight,
  CheckCircle,
  XCircle,
  AlertCircle,
  Clock,
  Zap,
} from "lucide-react";
import {
  useHookTypes,
  useHookHandlers,
  useHookExecutions,
  useHookStats,
  useTriggerHook,
  useEnableHookHandler,
  useDisableHookHandler,
} from "@/hooks/useHooks";
import { HookType, HookHandler } from "@/lib/tauri";
import { Spinner } from "@/components/Spinner/Spinner";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import styles from "./Hooks.module.css";
import clsx from "clsx";

type Tab = "types" | "handlers" | "executions";

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

export default function Hooks() {
  const [activeTab, setActiveTab] = useState<Tab>("types");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedHookType, setSelectedHookType] = useState<string | undefined>();
  const [triggerModal, setTriggerModal] = useState<HookType | null>(null);
  const [triggerData, setTriggerData] = useState("");

  const { data: hookTypes, isLoading: typesLoading, refetch: refetchTypes } = useHookTypes();
  const { data: handlers, isLoading: handlersLoading, refetch: refetchHandlers } = useHookHandlers(selectedHookType);
  const { data: executions, isLoading: executionsLoading, refetch: refetchExecutions } = useHookExecutions(
    selectedHookType,
    100
  );
  const { data: stats } = useHookStats();

  const triggerMutation = useTriggerHook();
  const enableMutation = useEnableHookHandler();
  const disableMutation = useDisableHookHandler();

  const isLoading =
    (activeTab === "types" && typesLoading) ||
    (activeTab === "handlers" && handlersLoading) ||
    (activeTab === "executions" && executionsLoading);

  const handleRefresh = () => {
    if (activeTab === "types") refetchTypes();
    else if (activeTab === "handlers") refetchHandlers();
    else refetchExecutions();
  };

  const handleToggleHandler = async (handler: HookHandler) => {
    if (handler.enabled) {
      await disableMutation.mutateAsync(handler.id);
    } else {
      await enableMutation.mutateAsync(handler.id);
    }
    refetchHandlers();
  };

  const handleTrigger = async () => {
    if (!triggerModal) return;
    try {
      let data = undefined;
      if (triggerData) {
        data = JSON.parse(triggerData);
      }
      await triggerMutation.mutateAsync({
        hook_name: triggerModal.name,
        data,
        source: "manual",
      });
      setTriggerModal(null);
      setTriggerData("");
      refetchExecutions();
      refetchTypes();
    } catch (error) {
      alert(`Failed to trigger hook: ${error}`);
    }
  };

  const filteredHookTypes = hookTypes?.filter((ht) => {
    if (!searchQuery) return true;
    return ht.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      ht.description.toLowerCase().includes(searchQuery.toLowerCase());
  });

  const filteredHandlers = handlers?.filter((h) => {
    if (!searchQuery) return true;
    return h.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      h.hook_type.toLowerCase().includes(searchQuery.toLowerCase());
  });

  const filteredExecutions = executions?.filter((e) => {
    if (!searchQuery) return true;
    return e.handler_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      e.hook_type.toLowerCase().includes(searchQuery.toLowerCase());
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "success":
        return <CheckCircle size={16} className={styles.statusSuccess} />;
      case "failed":
        return <XCircle size={16} className={styles.statusFailed} />;
      case "aborted":
        return <AlertCircle size={16} className={styles.statusAborted} />;
      default:
        return <Clock size={16} className={styles.statusPending} />;
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Hooks</h1>
        <button
          className="btn btn-secondary"
          onClick={handleRefresh}
          disabled={isLoading}
        >
          <RefreshCw size={16} className={isLoading ? styles.spinning : ""} />
          Refresh
        </button>
      </div>

      <div className={styles.stats}>
        <div className={styles.statCard}>
          <span className={styles.statValue}>{stats?.total_hook_types ?? 0}</span>
          <span className={styles.statLabel}>Hook Types</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statValue}>{stats?.active_handlers ?? 0}</span>
          <span className={styles.statLabel}>Active Handlers</span>
        </div>
        <div className={styles.statCard}>
          <span className={styles.statValue}>{stats?.total_executions ?? 0}</span>
          <span className={styles.statLabel}>Total Executions</span>
        </div>
        <div className={styles.statCard}>
          <span className={clsx(styles.statValue, (stats?.failed_executions ?? 0) > 0 && styles.statError)}>
            {stats?.failed_executions ?? 0}
          </span>
          <span className={styles.statLabel}>Failed</span>
        </div>
      </div>

      <div className={styles.tabs}>
        <button
          className={clsx(styles.tab, activeTab === "types" && styles.active)}
          onClick={() => {
            setActiveTab("types");
            setSelectedHookType(undefined);
          }}
        >
          <Webhook size={16} />
          By Type
          <span className={styles.tabCount}>{hookTypes?.length ?? 0}</span>
        </button>
        <button
          className={clsx(styles.tab, activeTab === "handlers" && styles.active)}
          onClick={() => setActiveTab("handlers")}
        >
          <ListTree size={16} />
          Handlers
          <span className={styles.tabCount}>{handlers?.length ?? 0}</span>
        </button>
        <button
          className={clsx(styles.tab, activeTab === "executions" && styles.active)}
          onClick={() => setActiveTab("executions")}
        >
          <History size={16} />
          Execution Log
        </button>
      </div>

      <div className={styles.filters}>
        <div className={styles.searchWrapper}>
          <Search size={16} className={styles.searchIcon} />
          <input
            type="text"
            placeholder={
              activeTab === "types"
                ? "Search hook types..."
                : activeTab === "handlers"
                ? "Search handlers..."
                : "Search executions..."
            }
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className={styles.searchInput}
          />
        </div>
        {(activeTab === "handlers" || activeTab === "executions") && (
          <select
            value={selectedHookType ?? ""}
            onChange={(e) => setSelectedHookType(e.target.value || undefined)}
            className={styles.filterSelect}
          >
            <option value="">All Hook Types</option>
            {hookTypes?.map((ht) => (
              <option key={ht.name} value={ht.name}>
                {ht.name}
              </option>
            ))}
          </select>
        )}
      </div>

      <div className={styles.content}>
        {isLoading ? (
          <div className={styles.loading}>
            <Spinner />
            <span>Loading...</span>
          </div>
        ) : activeTab === "types" ? (
          filteredHookTypes && filteredHookTypes.length > 0 ? (
            <div className={styles.hookTypeList}>
              {filteredHookTypes.map((hookType) => (
                <div key={hookType.name} className={styles.hookTypeCard}>
                  <div className={styles.hookTypeHeader}>
                    <div className={styles.hookTypeMeta}>
                      <Zap size={16} className={styles.hookTypeIcon} />
                      <div>
                        <h3 className={styles.hookTypeName}>{hookType.name}</h3>
                        <span className={styles.hookTypeDataType}>
                          Data: {hookType.data_type}
                        </span>
                      </div>
                    </div>
                    <div className={styles.hookTypeActions}>
                      <button
                        className={clsx(styles.actionBtn, styles.trigger)}
                        onClick={() => setTriggerModal(hookType)}
                        title="Trigger manually"
                      >
                        <Play size={16} />
                      </button>
                      <button
                        className={styles.actionBtn}
                        onClick={() => {
                          setSelectedHookType(hookType.name);
                          setActiveTab("handlers");
                        }}
                        title="View handlers"
                      >
                        <ChevronRight size={16} />
                      </button>
                    </div>
                  </div>
                  <p className={styles.hookTypeDescription}>{hookType.description}</p>
                  <div className={styles.hookTypeFooter}>
                    <span className={styles.handlerCount}>
                      {hookType.handler_count} handler{hookType.handler_count !== 1 ? "s" : ""}
                    </span>
                    {hookType.last_triggered && (
                      <span className={styles.lastTriggered}>
                        Last: {formatRelativeTime(hookType.last_triggered)}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState
              icon={Webhook}
              title="No hooks configured"
              description="Hooks automate actions on events"
            />
          )
        ) : activeTab === "handlers" ? (
          filteredHandlers && filteredHandlers.length > 0 ? (
            <div className={styles.handlerList}>
              {filteredHandlers.map((handler) => (
                <div key={handler.id} className={styles.handlerCard}>
                  <div className={styles.handlerHeader}>
                    <div className={styles.handlerMeta}>
                      {handler.enabled ? (
                        <CheckCircle size={16} className={styles.statusEnabled} />
                      ) : (
                        <AlertCircle size={16} className={styles.statusDisabled} />
                      )}
                      <div>
                        <h3 className={styles.handlerName}>{handler.name}</h3>
                        <span className={styles.handlerHookType}>
                          {handler.hook_type}
                        </span>
                      </div>
                    </div>
                    <div className={styles.handlerActions}>
                      <button
                        className={clsx(styles.actionBtn, handler.enabled && styles.enabled)}
                        onClick={() => handleToggleHandler(handler)}
                        title={handler.enabled ? "Disable" : "Enable"}
                      >
                        {handler.enabled ? <Power size={16} /> : <PowerOff size={16} />}
                      </button>
                    </div>
                  </div>
                  {handler.description && (
                    <p className={styles.handlerDescription}>{handler.description}</p>
                  )}
                  <div className={styles.handlerFooter}>
                    <span className={styles.handlerPriority}>
                      Priority: {handler.priority}
                    </span>
                    {handler.plugin_id && (
                      <span className={styles.handlerPlugin}>
                        Plugin: {handler.plugin_id}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState
              icon={ListTree}
              title="No handlers found"
              description={
                selectedHookType
                  ? `No handlers registered for ${selectedHookType}`
                  : "No handlers registered"
              }
            />
          )
        ) : filteredExecutions && filteredExecutions.length > 0 ? (
          <div className={styles.executionList}>
            <div className={styles.executionTable}>
              <div className={styles.executionHeader}>
                <span>Status</span>
                <span>Handler</span>
                <span>Hook Type</span>
                <span>Duration</span>
                <span>Timestamp</span>
              </div>
              {filteredExecutions.map((execution) => (
                <div key={execution.id} className={styles.executionRow}>
                  <span className={styles.executionStatus}>
                    {getStatusIcon(execution.status)}
                    {execution.status}
                  </span>
                  <span className={styles.executionHandler}>
                    {execution.handler_name}
                  </span>
                  <span className={styles.executionHookType}>
                    {execution.hook_type}
                  </span>
                  <span className={styles.executionDuration}>
                    {execution.duration_ms !== undefined ? `${execution.duration_ms}ms` : "-"}
                  </span>
                  <span className={styles.executionTimestamp}>
                    {formatRelativeTime(execution.started_at)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <EmptyState
            icon={History}
            title="No executions found"
            description="Hook executions will appear here"
          />
        )}
      </div>

      {triggerModal && (
        <div className={styles.modalOverlay} onClick={() => setTriggerModal(null)}>
          <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
            <h2 className={styles.modalTitle}>Trigger Hook</h2>
            <p className={styles.modalDescription}>
              Manually trigger <strong>{triggerModal.name}</strong> for testing.
            </p>
            <div className={styles.inputGroup}>
              <label className={styles.inputLabel}>
                Data (JSON, optional)
              </label>
              <textarea
                placeholder='{"key": "value"}'
                value={triggerData}
                onChange={(e) => setTriggerData(e.target.value)}
                className={styles.modalTextarea}
                rows={4}
              />
            </div>
            <div className={styles.modalActions}>
              <button
                className="btn btn-secondary"
                onClick={() => setTriggerModal(null)}
              >
                Cancel
              </button>
              <button
                className="btn btn-primary"
                onClick={handleTrigger}
                disabled={triggerMutation.isPending}
              >
                {triggerMutation.isPending ? "Triggering..." : "Trigger"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
