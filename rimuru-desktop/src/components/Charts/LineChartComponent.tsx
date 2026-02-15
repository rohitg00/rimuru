import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";

interface LineChartData {
  label: string;
  value: number;
}

interface LineChartComponentProps {
  data: LineChartData[];
  height?: number;
  lineColor?: string;
  strokeWidth?: number;
  showDots?: boolean;
  formatValue?: (value: number) => string;
  formatYAxis?: (value: number) => string;
}

export default function LineChartComponent({
  data,
  height = 300,
  lineColor = "var(--color-accent)",
  strokeWidth = 2,
  showDots = false,
  formatValue = (v) => v.toLocaleString(),
  formatYAxis,
}: LineChartComponentProps) {
  if (!data || data.length === 0) {
    return null;
  }

  return (
    <ResponsiveContainer width="100%" height={height}>
      <LineChart data={data}>
        <XAxis
          dataKey="label"
          tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
          axisLine={false}
        />
        <YAxis
          tick={{ fontSize: 11, fill: "var(--color-text-muted)" }}
          axisLine={false}
          tickFormatter={formatYAxis}
        />
        <Tooltip
          formatter={(value: number) => formatValue(value)}
          contentStyle={{
            background: "var(--color-bg-secondary)",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--radius-sm)",
          }}
        />
        <Line
          type="monotone"
          dataKey="value"
          stroke={lineColor}
          strokeWidth={strokeWidth}
          dot={showDots}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
