import { useEffect, useState } from "react";
import { Play, Square, DollarSign } from "lucide-react";
import { events, SessionEventPayload, CostRecordedPayload } from "@/lib/tauri";
import styles from "./ActivityFeed.module.css";

interface ActivityItem {
  id: string;
  type: "session_started" | "session_ended" | "cost_recorded";
  message: string;
  timestamp: string;
}

export default function ActivityFeed() {
  const [activities, setActivities] = useState<ActivityItem[]>([]);

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    events.onSessionStarted((payload: SessionEventPayload) => {
      setActivities((prev) => [
        {
          id: `${payload.session_id}-started`,
          type: "session_started",
          message: `Session started`,
          timestamp: payload.timestamp,
        },
        ...prev.slice(0, 9),
      ]);
    }).then((fn) => unlisteners.push(fn));

    events.onSessionEnded((payload: SessionEventPayload) => {
      setActivities((prev) => [
        {
          id: `${payload.session_id}-ended`,
          type: "session_ended",
          message: `Session ended`,
          timestamp: payload.timestamp,
        },
        ...prev.slice(0, 9),
      ]);
    }).then((fn) => unlisteners.push(fn));

    events.onCostRecorded((payload: CostRecordedPayload) => {
      setActivities((prev) => [
        {
          id: `${payload.session_id}-cost-${Date.now()}`,
          type: "cost_recorded",
          message: `Cost recorded: $${payload.cost.toFixed(4)} for ${payload.model}`,
          timestamp: payload.timestamp,
        },
        ...prev.slice(0, 9),
      ]);
    }).then((fn) => unlisteners.push(fn));

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  const getIcon = (type: ActivityItem["type"]) => {
    switch (type) {
      case "session_started":
        return <Play size={14} className={styles.iconSuccess} />;
      case "session_ended":
        return <Square size={14} className={styles.iconWarning} />;
      case "cost_recorded":
        return <DollarSign size={14} className={styles.iconInfo} />;
    }
  };

  if (activities.length === 0) {
    return <p className={styles.empty}>No recent activity</p>;
  }

  return (
    <div className={styles.feed}>
      {activities.map((activity) => (
        <div key={activity.id} className={styles.item}>
          <div className={styles.icon}>{getIcon(activity.type)}</div>
          <div className={styles.content}>
            <span className={styles.message}>{activity.message}</span>
            <span className={styles.time}>
              {new Date(activity.timestamp).toLocaleTimeString()}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}
