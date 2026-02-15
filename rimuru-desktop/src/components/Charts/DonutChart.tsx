import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from "recharts";

const COLORS = [
  "var(--color-accent)",
  "var(--color-success)",
  "var(--color-warning)",
  "var(--color-error)",
  "var(--color-info)",
];

interface DonutChartData {
  name: string;
  value: number;
  percentage?: number;
}

interface DonutChartProps {
  data: DonutChartData[];
  height?: number;
  innerRadius?: number;
  outerRadius?: number;
  showLabel?: boolean;
  formatValue?: (value: number) => string;
}

export default function DonutChart({
  data,
  height = 250,
  innerRadius = 40,
  outerRadius = 80,
  showLabel = true,
  formatValue = (v) => v.toLocaleString(),
}: DonutChartProps) {
  if (!data || data.length === 0) {
    return null;
  }

  return (
    <ResponsiveContainer width="100%" height={height}>
      <PieChart>
        <Pie
          data={data}
          dataKey="value"
          nameKey="name"
          cx="50%"
          cy="50%"
          innerRadius={innerRadius}
          outerRadius={outerRadius}
          label={
            showLabel
              ? ({ name, percentage }) =>
                  `${name} (${(percentage ?? 0).toFixed(0)}%)`
              : false
          }
        >
          {data.map((_, index) => (
            <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
          ))}
        </Pie>
        <Tooltip
          formatter={(value: number) => formatValue(value)}
          contentStyle={{
            background: "var(--color-bg-secondary)",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--radius-sm)",
          }}
        />
      </PieChart>
    </ResponsiveContainer>
  );
}
