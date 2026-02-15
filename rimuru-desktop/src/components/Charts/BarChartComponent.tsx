import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";

interface BarChartData {
  name: string;
  value: number;
}

interface BarChartComponentProps {
  data: BarChartData[];
  height?: number;
  barColor?: string;
  formatValue?: (value: number) => string;
}

export default function BarChartComponent({
  data,
  height = 250,
  barColor = "var(--color-accent)",
  formatValue = (v) => v.toLocaleString(),
}: BarChartComponentProps) {
  if (!data || data.length === 0) {
    return null;
  }

  return (
    <ResponsiveContainer width="100%" height={height}>
      <BarChart data={data}>
        <XAxis
          dataKey="name"
          tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
          axisLine={false}
        />
        <YAxis
          tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
          axisLine={false}
        />
        <Tooltip
          formatter={(value: number) => formatValue(value)}
          contentStyle={{
            background: "var(--color-bg-secondary)",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--radius-sm)",
          }}
        />
        <Bar dataKey="value" fill={barColor} radius={[4, 4, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  );
}
