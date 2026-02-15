import styles from "./SettingsModal.module.css";

export default function AICommandsTab() {
  return (
    <div className={styles.placeholder}>
      <p>Custom AI Commands</p>
      <p>Coming soon</p>
      <button disabled style={{ marginTop: 12, padding: "6px 16px", fontSize: 13, borderRadius: 6, border: "1px solid var(--color-border)", background: "var(--color-bg-tertiary)", color: "var(--color-text-muted)", cursor: "not-allowed" }}>
        Add Command
      </button>
    </div>
  );
}
