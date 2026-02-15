import { LucideIcon, TrendingUp, TrendingDown, Minus } from "lucide-react";
import styles from "./StatsCard.module.css";
import clsx from "clsx";

interface StatsCardProps {
  icon: LucideIcon;
  label: string;
  value: string | number;
  trend?: {
    value: number;
    direction: "up" | "down" | "neutral";
  };
}

export default function StatsCard({ icon: Icon, label, value, trend }: StatsCardProps) {
  const TrendIcon = trend?.direction === "up" ? TrendingUp : trend?.direction === "down" ? TrendingDown : Minus;

  return (
    <div className={styles.card}>
      <div className={styles.iconWrapper}>
        <Icon size={20} />
      </div>
      <div className={styles.content}>
        <span className={styles.label}>{label}</span>
        <span className={styles.value}>{value}</span>
      </div>
      {trend && (
        <div className={clsx(styles.trend, styles[trend.direction])}>
          <TrendIcon size={14} />
        </div>
      )}
    </div>
  );
}
