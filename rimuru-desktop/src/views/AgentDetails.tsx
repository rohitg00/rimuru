import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, RefreshCw, Clock, DollarSign } from "lucide-react";
import { useAgent } from "@/hooks/useAgents";
import { useSessions } from "@/hooks/useSessions";
import styles from "./AgentDetails.module.css";

function statusBadgeClass(status: string): string {
  if (status === "active") return "badge-success";
  if (status === "completed") return "badge-primary";
  return "badge-warning";
}

export default function AgentDetails() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: agent, isLoading } = useAgent(id ?? "");
  const { data: sessions } = useSessions({ agent_id: id, limit: 10 });

  if (isLoading) {
    return <div className={styles.loading}>Loading agent details...</div>;
  }

  if (!agent) {
    return (
      <div className={styles.notFound}>
        <p>Agent not found</p>
        <button className="btn btn-primary" onClick={() => navigate("/agents")}>
          Back to Agents
        </button>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <button className={styles.backBtn} onClick={() => navigate("/agents")}>
        <ArrowLeft size={16} />
        Back to Agents
      </button>

      <div className={styles.header}>
        <div className={styles.info}>
          <h1 className={styles.name}>{agent.agent.name}</h1>
          <span className={styles.type}>{agent.agent.agent_type}</span>
        </div>
        <div className={styles.actions}>
          <span
            className={`badge ${agent.is_active ? "badge-success" : "badge-warning"}`}
          >
            {agent.is_active ? "Active" : "Idle"}
          </span>
          <button className="btn btn-secondary">
            <RefreshCw size={16} />
            Reconnect
          </button>
        </div>
      </div>

      <div className={styles.grid}>
        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Configuration</h3>
          <div className={styles.configItem}>
            <span className={styles.configLabel}>ID</span>
            <code className={styles.configValue}>{agent.agent.id}</code>
          </div>
          <div className={styles.configItem}>
            <span className={styles.configLabel}>Type</span>
            <span className={styles.configValue}>{agent.agent.agent_type}</span>
          </div>
          <div className={styles.configItem}>
            <span className={styles.configLabel}>Config Path</span>
            <code className={styles.configValue}>
              {agent.agent.config_path ?? "Not set"}
            </code>
          </div>
          <div className={styles.configItem}>
            <span className={styles.configLabel}>Created</span>
            <span className={styles.configValue}>
              {new Date(agent.agent.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>

        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Statistics</h3>
          <div className={styles.statRow}>
            <Clock size={16} />
            <span>Total Sessions: {agent.session_count}</span>
          </div>
          <div className={styles.statRow}>
            <DollarSign size={16} />
            <span>Total Cost: ${agent.total_cost?.toFixed(2) ?? "0.00"}</span>
          </div>
        </div>
      </div>

      <div className={styles.card}>
        <h3 className={styles.cardTitle}>Recent Sessions</h3>
        {sessions && sessions.length > 0 ? (
          <ul className={styles.sessionList}>
            {sessions.map((session) => (
              <li key={session.id} className={styles.sessionItem}>
                <span
                  className={`badge ${statusBadgeClass(session.status)}`}
                >
                  {session.status}
                </span>
                <span>{new Date(session.started_at).toLocaleString()}</span>
                {session.duration_secs != null && (
                  <span>
                    {Math.floor(session.duration_secs / 60)}m {session.duration_secs % 60}s
                  </span>
                )}
                <span>${session.total_cost.toFixed(4)}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>No recent sessions</p>
        )}
      </div>
    </div>
  );
}
