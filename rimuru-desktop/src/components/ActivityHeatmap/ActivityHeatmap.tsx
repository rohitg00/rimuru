import { useMemo } from "react";
import styles from "./ActivityHeatmap.module.css";

interface HeatmapData {
  date: string;
  count: number;
}

interface ActivityHeatmapProps {
  data: HeatmapData[];
}

const DAYS = ["", "Mon", "", "Wed", "", "Fri", ""];
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

function getIntensity(count: number): number {
  if (count === 0) return 0;
  if (count <= 3) return 1;
  if (count <= 7) return 2;
  if (count <= 15) return 3;
  return 4;
}

export default function ActivityHeatmap({ data }: ActivityHeatmapProps) {
  const grid = useMemo(() => {
    const map = new Map<string, number>();
    for (const d of data) {
      map.set(d.date, d.count);
    }

    const today = new Date();
    const cells: Array<{ date: string; count: number; col: number; row: number }> = [];

    for (let week = 51; week >= 0; week--) {
      for (let day = 0; day < 7; day++) {
        const d = new Date(today);
        d.setDate(d.getDate() - (week * 7 + (6 - day)));
        const dateStr = d.toISOString().split("T")[0];
        const count = map.get(dateStr) ?? 0;
        cells.push({ date: dateStr, count, col: 51 - week, row: day });
      }
    }

    return cells;
  }, [data]);

  const monthLabels = useMemo(() => {
    const labels: Array<{ month: string; col: number }> = [];
    let lastMonth = -1;
    for (const cell of grid) {
      if (cell.row !== 0) continue;
      const month = new Date(cell.date).getMonth();
      if (month !== lastMonth) {
        labels.push({ month: MONTHS[month], col: cell.col });
        lastMonth = month;
      }
    }
    return labels;
  }, [grid]);

  return (
    <div className={styles.container}>
      <div className={styles.months}>
        {monthLabels.map((l, i) => (
          <span key={i} className={styles.monthLabel} style={{ gridColumn: l.col + 2 }}>
            {l.month}
          </span>
        ))}
      </div>
      <div className={styles.grid}>
        <div className={styles.dayLabels}>
          {DAYS.map((d, i) => (
            <span key={i} className={styles.dayLabel}>{d}</span>
          ))}
        </div>
        <div className={styles.cells}>
          {grid.map((cell, i) => (
            <div
              key={i}
              className={`${styles.cell} ${styles[`intensity${getIntensity(cell.count)}`]}`}
              title={`${cell.date}: ${cell.count} actions`}
              style={{ gridColumn: cell.col + 1, gridRow: cell.row + 1 }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
