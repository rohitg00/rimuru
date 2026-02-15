import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Clock, DollarSign, Hash } from "lucide-react";
import { useSession } from "@/hooks/useSessions";
import styles from "./SessionDetails.module.css";

export default function SessionDetails() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: session, isLoading } = useSession(id ?? "");

  if (isLoading) {
    return <div className={styles.loading}>Loading session details...</div>;
  }

  if (!session) {
    return (
      <div className={styles.notFound}>
        <p>Session not found</p>
        <button className="btn btn-primary" onClick={() => navigate("/sessions")}>
          Back to Sessions
        </button>
      </div>
    );
  }

  const formatDuration = (secs?: number) => {
    if (!secs) return "--";
    const mins = Math.floor(secs / 60);
    const remainingSecs = secs % 60;
    return `${mins}m ${remainingSecs}s`;
  };

  return (
    <div className={styles.container}>
      <button className={styles.backBtn} onClick={() => navigate("/sessions")}>
        <ArrowLeft size={16} />
        Back to Sessions
      </button>

      <div className={styles.header}>
        <div className={styles.info}>
          <h1 className={styles.title}>Session Details</h1>
          <code className={styles.sessionId}>{session.id}</code>
        </div>
        <span
          className={`badge ${
            session.status === "Active"
              ? "badge-success"
              : session.status === "Completed"
              ? "badge-info"
              : "badge-error"
          }`}
        >
          {session.status}
        </span>
      </div>

      <div className={styles.grid}>
        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Timing</h3>
          <div className={styles.statItem}>
            <Clock size={16} />
            <div>
              <span className={styles.statLabel}>Started</span>
              <span className={styles.statValue}>
                {new Date(session.started_at).toLocaleString()}
              </span>
            </div>
          </div>
          <div className={styles.statItem}>
            <Clock size={16} />
            <div>
              <span className={styles.statLabel}>Ended</span>
              <span className={styles.statValue}>
                {session.ended_at
                  ? new Date(session.ended_at).toLocaleString()
                  : "In progress"}
              </span>
            </div>
          </div>
          <div className={styles.statItem}>
            <Clock size={16} />
            <div>
              <span className={styles.statLabel}>Duration</span>
              <span className={styles.statValue}>
                {formatDuration(session.duration_secs)}
              </span>
            </div>
          </div>
        </div>

        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Usage</h3>
          <div className={styles.statItem}>
            <Hash size={16} />
            <div>
              <span className={styles.statLabel}>Total Tokens</span>
              <span className={styles.statValue}>
                {session.total_tokens.toLocaleString()}
              </span>
            </div>
          </div>
          <div className={styles.statItem}>
            <DollarSign size={16} />
            <div>
              <span className={styles.statLabel}>Total Cost</span>
              <span className={styles.statValue}>
                ${session.total_cost.toFixed(4)}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
