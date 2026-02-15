import styles from "./LaunchWizard.module.css";

interface ProviderCardProps {
  name: string;
  icon: string;
  status: "installed" | "not-installed" | "coming-soon";
  selected: boolean;
  onClick: () => void;
}

export default function ProviderCard({ name, icon, status, selected, onClick }: ProviderCardProps) {
  const disabled = status === "coming-soon";
  return (
    <button
      className={`${styles.providerCard} ${selected ? styles.providerSelected : ""} ${disabled ? styles.providerDisabled : ""}`}
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
    >
      <span className={styles.providerIcon}>{icon}</span>
      <span className={styles.providerName}>{name}</span>
      <span className={`${styles.providerStatus} ${styles[`status_${status.replace("-", "_")}`] || ""}`}>
        {status === "installed" ? "Installed" : status === "not-installed" ? "Not Installed" : "Coming Soon"}
      </span>
    </button>
  );
}
