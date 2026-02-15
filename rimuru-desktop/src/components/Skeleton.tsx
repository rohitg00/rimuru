import clsx from "clsx";
import styles from "./Skeleton.module.css";

interface SkeletonProps {
  variant?: "text" | "rectangular" | "circular" | "card" | "stats";
  width?: string | number;
  height?: string | number;
  className?: string;
  animation?: "pulse" | "wave" | "none";
}

export function Skeleton({
  variant = "text",
  width,
  height,
  className,
  animation = "pulse",
}: SkeletonProps) {
  const style: React.CSSProperties = {
    width: typeof width === "number" ? `${width}px` : width,
    height: typeof height === "number" ? `${height}px` : height,
  };

  return (
    <div
      className={clsx(
        styles.skeleton,
        styles[variant],
        styles[animation],
        className
      )}
      style={style}
      aria-hidden="true"
    />
  );
}

export function SkeletonText({
  lines = 1,
  width = "100%",
  className,
}: {
  lines?: number;
  width?: string | number;
  className?: string;
}) {
  return (
    <div className={clsx(styles.textContainer, className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          variant="text"
          width={i === lines - 1 && lines > 1 ? "70%" : width}
        />
      ))}
    </div>
  );
}

export function StatsCardSkeleton() {
  return (
    <div className={styles.statsCard}>
      <div className={styles.statsHeader}>
        <Skeleton variant="circular" width={40} height={40} />
        <div className={styles.statsText}>
          <Skeleton variant="text" width={80} height={14} />
          <Skeleton variant="text" width={60} height={24} />
        </div>
      </div>
      <Skeleton variant="text" width={100} height={14} />
    </div>
  );
}

export function ChartCardSkeleton() {
  return (
    <div className={styles.chartCard}>
      <Skeleton variant="text" width={120} height={18} />
      <div className={styles.chartPlaceholder}>
        <Skeleton variant="rectangular" width="100%" height={160} />
      </div>
    </div>
  );
}

export function TableRowSkeleton({ columns = 4 }: { columns?: number }) {
  return (
    <div className={styles.tableRow}>
      {Array.from({ length: columns }).map((_, i) => (
        <Skeleton
          key={i}
          variant="text"
          width={i === 0 ? 200 : i === columns - 1 ? 80 : 120}
          height={16}
        />
      ))}
    </div>
  );
}

export function TableSkeleton({
  rows = 5,
  columns = 4,
}: {
  rows?: number;
  columns?: number;
}) {
  return (
    <div className={styles.table}>
      <div className={styles.tableHeader}>
        {Array.from({ length: columns }).map((_, i) => (
          <Skeleton key={i} variant="text" width={80} height={14} />
        ))}
      </div>
      {Array.from({ length: rows }).map((_, i) => (
        <TableRowSkeleton key={i} columns={columns} />
      ))}
    </div>
  );
}

export function AgentCardSkeleton() {
  return (
    <div className={styles.agentCard}>
      <div className={styles.agentHeader}>
        <Skeleton variant="circular" width={48} height={48} />
        <div className={styles.agentInfo}>
          <Skeleton variant="text" width={140} height={18} />
          <Skeleton variant="text" width={100} height={14} />
        </div>
      </div>
      <div className={styles.agentStats}>
        <Skeleton variant="rectangular" width="100%" height={60} />
      </div>
      <div className={styles.agentActions}>
        <Skeleton variant="rectangular" width={80} height={32} />
        <Skeleton variant="rectangular" width={80} height={32} />
      </div>
    </div>
  );
}

export function ListItemSkeleton() {
  return (
    <div className={styles.listItem}>
      <Skeleton variant="circular" width={32} height={32} />
      <div className={styles.listItemContent}>
        <Skeleton variant="text" width={160} height={16} />
        <Skeleton variant="text" width={100} height={12} />
      </div>
      <Skeleton variant="rectangular" width={60} height={24} />
    </div>
  );
}

export function DashboardSkeleton() {
  return (
    <div className={styles.dashboard}>
      <Skeleton variant="text" width={200} height={32} className={styles.title} />
      <div className={styles.statsGrid}>
        <StatsCardSkeleton />
        <StatsCardSkeleton />
        <StatsCardSkeleton />
        <StatsCardSkeleton />
      </div>
      <div className={styles.chartsRow}>
        <ChartCardSkeleton />
        <ChartCardSkeleton />
      </div>
    </div>
  );
}
