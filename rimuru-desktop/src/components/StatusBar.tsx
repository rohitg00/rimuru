import { Cpu, HardDrive, Clock } from "lucide-react";
import { useRealtimeMetrics } from "@/hooks/useMetrics";
import styles from "./StatusBar.module.css";

export default function StatusBar() {
  const metrics = useRealtimeMetrics();

  return (
    <footer className={styles.statusBar}>
      <div className={styles.item}>
        <Cpu size={14} />
        <span>CPU: {metrics?.cpu_usage?.toFixed(1) ?? "--"}%</span>
      </div>

      <div className={styles.item}>
        <HardDrive size={14} />
        <span>
          Memory: {metrics?.memory_used_mb ?? "--"} MB (
          {metrics?.memory_usage_percent?.toFixed(1) ?? "--"}%)
        </span>
      </div>

      <div className={styles.item}>
        <Clock size={14} />
        <span>Active Sessions: {metrics?.active_sessions ?? 0}</span>
      </div>

      <div className={styles.spacer} />

      <div className={styles.item}>
        <span className={styles.timestamp}>
          {metrics?.timestamp
            ? new Date(metrics.timestamp).toLocaleTimeString()
            : "--:--:--"}
        </span>
      </div>
    </footer>
  );
}
