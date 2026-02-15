import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Download, Clock } from "lucide-react";
import { useSessions } from "@/hooks/useSessions";
import { Spinner } from "@/components/Spinner/Spinner";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import { commands } from "@/lib/tauri";
import DataTable from "@/components/DataTable";
import { Session } from "@/lib/tauri";
import styles from "./Sessions.module.css";

type SessionColumn = {
  key: keyof Session;
  label: string;
  width?: string;
  render?: (value: Session[keyof Session], row: Session) => React.ReactNode;
};

export default function Sessions() {
  const navigate = useNavigate();
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const { data: sessions, isLoading } = useSessions(
    statusFilter !== "all" ? { status: statusFilter } : undefined
  );

  const formatDuration = (secs: Session[keyof Session]) => {
    if (typeof secs !== "number" || !secs) return "--";
    const mins = Math.floor(secs / 60);
    const remainingSecs = secs % 60;
    return `${mins}m ${remainingSecs}s`;
  };

  const columns: SessionColumn[] = [
    { key: "id", label: "ID", width: "200px" },
    { key: "status", label: "Status", width: "100px" },
    {
      key: "started_at",
      label: "Started",
      render: (value) => (typeof value === "string" ? new Date(value).toLocaleString() : "--"),
    },
    {
      key: "duration_secs",
      label: "Duration",
      render: formatDuration,
    },
    {
      key: "total_tokens",
      label: "Tokens",
      render: (value) => (typeof value === "number" ? value.toLocaleString() : "--"),
    },
    {
      key: "total_cost",
      label: "Cost",
      render: (value) => (typeof value === "number" ? `$${value.toFixed(4)}` : "--"),
    },
  ];

  const [exporting, setExporting] = useState(false);

  const handleExport = async (format: "csv" | "json" = "csv") => {
    setExporting(true);
    try {
      const filters = statusFilter !== "all" ? { status: statusFilter } : undefined;
      const data = await commands.exportSessions(format, filters);
      const blob = new Blob([data], { type: format === "csv" ? "text/csv" : "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `sessions-export.${format}`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error("Export failed:", e);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Sessions</h1>
        <div className={styles.actions}>
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className={styles.filterSelect}
          >
            <option value="all">All Status</option>
            <option value="active">Active</option>
            <option value="completed">Completed</option>
            <option value="failed">Failed</option>
          </select>
          <button className="btn btn-secondary" onClick={() => handleExport("csv")} disabled={exporting}>
            <Download size={16} />
            Export CSV
          </button>
          <button className="btn btn-secondary" onClick={() => handleExport("json")} disabled={exporting}>
            <Download size={16} />
            Export JSON
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className={styles.loading}><Spinner /> Loading sessions...</div>
      ) : sessions && sessions.length > 0 ? (
        <DataTable
          data={sessions}
          columns={columns}
          onRowClick={(row) => navigate(`/sessions/${row.id}`)}
        />
      ) : (
        <EmptyState
          icon={Clock}
          title="No sessions yet"
          description={
            statusFilter !== "all"
              ? `No ${statusFilter} sessions. Try selecting a different status filter.`
              : "Launch an agent to start your first session"
          }
        />
      )}
    </div>
  );
}
