import {
  ResponsiveContainer,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
} from "recharts";

interface MetricsChartProps {
  data: { time: string; value: number }[];
  color: string;
  label: string;
  unit?: string;
  max?: number;
}

function MetricsTooltip({
  active,
  payload,
  label,
  unit,
}: {
  active?: boolean;
  payload?: { value: number }[];
  label?: string;
  unit?: string;
}) {
  if (!active || !payload?.length) return null;
  return (
    <div className="rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)] px-3 py-2 shadow-xl">
      <p className="text-xs text-[var(--text-secondary)]">{label}</p>
      <p className="text-sm font-semibold text-[var(--text-primary)]">
        {payload[0]!.value.toFixed(1)}{unit ?? ""}
      </p>
    </div>
  );
}

export default function MetricsChart({ data, color, label, unit, max }: MetricsChartProps) {
  return (
    <div>
      <p className="text-xs font-medium text-[var(--text-secondary)] mb-2 uppercase tracking-wider">
        {label}
      </p>
      <ResponsiveContainer width="100%" height={180}>
        <AreaChart data={data} margin={{ top: 5, right: 5, bottom: 5, left: 5 }}>
          <defs>
            <linearGradient id={`grad-${label}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={color} stopOpacity={0.3} />
              <stop offset="95%" stopColor={color} stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis
            dataKey="time"
            tick={{ fill: "var(--text-secondary)", fontSize: 10 }}
            tickLine={false}
            axisLine={false}
          />
          <YAxis
            tick={{ fill: "var(--text-secondary)", fontSize: 10 }}
            tickLine={false}
            axisLine={false}
            domain={[0, max ?? "auto"]}
            tickFormatter={(v: number) => `${v}${unit ?? ""}`}
          />
          <Tooltip content={<MetricsTooltip unit={unit} />} />
          <Area
            type="monotone"
            dataKey="value"
            stroke={color}
            strokeWidth={2}
            fill={`url(#grad-${label})`}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

interface GaugeProps {
  value: number;
  max: number;
  label: string;
  unit?: string;
  color: string;
}

export function Gauge({ value, max, label, unit, color }: GaugeProps) {
  const pct = Math.min((value / max) * 100, 100);
  return (
    <div className="flex flex-col items-center">
      <div className="relative w-28 h-28">
        <svg viewBox="0 0 100 100" className="w-full h-full -rotate-90">
          <circle
            cx="50"
            cy="50"
            r="42"
            fill="none"
            stroke="var(--border)"
            strokeWidth="8"
          />
          <circle
            cx="50"
            cy="50"
            r="42"
            fill="none"
            stroke={color}
            strokeWidth="8"
            strokeDasharray={`${pct * 2.64} ${264 - pct * 2.64}`}
            strokeLinecap="round"
            className="transition-all duration-700"
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-lg font-bold text-[var(--text-primary)]">
            {value.toFixed(1)}
          </span>
          <span className="text-[10px] text-[var(--text-secondary)]">{unit}</span>
        </div>
      </div>
      <p className="mt-2 text-xs font-medium text-[var(--text-secondary)] uppercase tracking-wider">
        {label}
      </p>
    </div>
  );
}
