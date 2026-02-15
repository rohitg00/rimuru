import { useState, useEffect, useCallback } from "react";
import { load } from "@tauri-apps/plugin-store";
import type { Store } from "@tauri-apps/plugin-store";
import styles from "./SettingsModal.module.css";

interface NotificationSettings {
  desktopNotifications: boolean;
  sound: boolean;
  costThreshold: string;
}

export default function NotificationsTab() {
  const [desktopNotifications, setDesktopNotifications] = useState(true);
  const [sound, setSound] = useState(true);
  const [costThreshold, setCostThreshold] = useState("5.00");
  const [store, setStore] = useState<Store | null>(null);

  useEffect(() => {
    const init = async () => {
      try {
        const s = await load("settings.json", { defaults: {}, autoSave: true });
        setStore(s);
        const saved = await s.get<NotificationSettings>("notifications");
        if (saved) {
          setDesktopNotifications(saved.desktopNotifications ?? true);
          setSound(saved.sound ?? true);
          setCostThreshold(saved.costThreshold ?? "5.00");
        }
      } catch {
        // store may not be available
      }
    };
    init();
  }, []);

  const persist = useCallback(
    async (settings: NotificationSettings) => {
      if (!store) return;
      try {
        await store.set("notifications", settings);
        await store.save();
      } catch {
        // ignore persistence errors
      }
    },
    [store]
  );

  const handleDesktopNotifications = (val: boolean) => {
    setDesktopNotifications(val);
    persist({ desktopNotifications: val, sound, costThreshold });
  };

  const handleSound = (val: boolean) => {
    setSound(val);
    persist({ desktopNotifications, sound: val, costThreshold });
  };

  const handleCostThreshold = (val: string) => {
    setCostThreshold(val);
    persist({ desktopNotifications, sound, costThreshold: val });
  };

  return (
    <div>
      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Desktop Notifications</div>
          <div className={styles.settingDesc}>Show system notifications for agent events</div>
        </div>
        <button
          className={`${styles.toggle} ${desktopNotifications ? styles.toggleActive : ""}`}
          onClick={() => handleDesktopNotifications(!desktopNotifications)}
        >
          <div className={styles.toggleKnob} />
        </button>
      </div>

      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Sound</div>
          <div className={styles.settingDesc}>Play sound effects for notifications</div>
        </div>
        <button
          className={`${styles.toggle} ${sound ? styles.toggleActive : ""}`}
          onClick={() => handleSound(!sound)}
        >
          <div className={styles.toggleKnob} />
        </button>
      </div>

      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Cost Alert Threshold</div>
          <div className={styles.settingDesc}>Alert when session cost exceeds this amount ($)</div>
        </div>
        <input
          className={styles.textInput}
          type="text"
          value={costThreshold}
          onChange={(e) => handleCostThreshold(e.target.value)}
        />
      </div>
    </div>
  );
}
