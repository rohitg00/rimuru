import styles from "./Gauge.module.css";

interface GaugeProps {
  value: number;
  max: number;
  unit?: string;
  color?: string;
  size?: number;
}

export default function Gauge({
  value,
  max,
  unit = "",
  color = "var(--color-accent)",
  size = 120,
}: GaugeProps) {
  const percentage = Math.min((value / max) * 100, 100);
  const strokeWidth = 12;
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const strokeDasharray = circumference;
  const strokeDashoffset = circumference - (percentage / 100) * circumference * 0.75;

  return (
    <div className={styles.gauge} style={{ width: size, height: size }}>
      <svg viewBox={`0 0 ${size} ${size}`} className={styles.svg}>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--color-bg-tertiary)"
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeDasharray={strokeDasharray}
          strokeDashoffset={circumference * 0.25}
          transform={`rotate(135 ${size / 2} ${size / 2})`}
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={color}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeDasharray={strokeDasharray}
          strokeDashoffset={strokeDashoffset}
          transform={`rotate(135 ${size / 2} ${size / 2})`}
          className={styles.progress}
        />
      </svg>
      <div className={styles.value}>
        <span className={styles.number}>{value.toFixed(1)}</span>
        <span className={styles.unit}>{unit}</span>
      </div>
    </div>
  );
}
