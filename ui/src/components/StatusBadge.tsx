interface StatusBadgeProps {
  status: string;
  size?: "sm" | "md" | "lg";
}

const STATUS_STYLES: Record<string, { bg: string; text: string; dot: string }> = {
  connected: { bg: "bg-[var(--success)]/15", text: "text-[var(--success)]", dot: "bg-[var(--success)]" },
  active: { bg: "bg-[var(--success)]/15", text: "text-[var(--success)]", dot: "bg-[var(--success)]" },
  success: { bg: "bg-[var(--success)]/15", text: "text-[var(--success)]", dot: "bg-[var(--success)]" },
  enabled: { bg: "bg-[var(--success)]/15", text: "text-[var(--success)]", dot: "bg-[var(--success)]" },
  busy: { bg: "bg-[var(--warning)]/15", text: "text-[var(--warning)]", dot: "bg-[var(--warning)]" },
  paused: { bg: "bg-[var(--warning)]/15", text: "text-[var(--warning)]", dot: "bg-[var(--warning)]" },
  idle: { bg: "bg-[var(--text-secondary)]/10", text: "text-[var(--text-secondary)]", dot: "bg-[var(--text-secondary)]" },
  disconnected: { bg: "bg-[var(--error)]/15", text: "text-[var(--error)]", dot: "bg-[var(--error)]" },
  error: { bg: "bg-[var(--error)]/15", text: "text-[var(--error)]", dot: "bg-[var(--error)]" },
  failed: { bg: "bg-[var(--error)]/15", text: "text-[var(--error)]", dot: "bg-[var(--error)]" },
  timeout: { bg: "bg-[var(--error)]/15", text: "text-[var(--error)]", dot: "bg-[var(--error)]" },
  disabled: { bg: "bg-[var(--text-secondary)]/10", text: "text-[var(--text-secondary)]", dot: "bg-[var(--text-secondary)]" },
  completed: { bg: "bg-[var(--accent)]/15", text: "text-[var(--accent)]", dot: "bg-[var(--accent)]" },
  installed: { bg: "bg-[var(--accent)]/15", text: "text-[var(--accent)]", dot: "bg-[var(--accent)]" },
};

const SIZE_CLASSES = {
  sm: "text-xs px-1.5 py-0.5",
  md: "text-xs px-2 py-1",
  lg: "text-sm px-2.5 py-1",
};

const DOT_SIZES = {
  sm: "w-1.5 h-1.5",
  md: "w-2 h-2",
  lg: "w-2.5 h-2.5",
};

export default function StatusBadge({ status, size = "md" }: StatusBadgeProps) {
  const s = STATUS_STYLES[status] ?? STATUS_STYLES.idle!;
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full font-medium ${s.bg} ${s.text} ${SIZE_CLASSES[size]}`}
    >
      <span className={`${DOT_SIZES[size]} rounded-full ${s.dot} ${status === "connected" || status === "active" || status === "busy" ? "animate-pulse" : ""}`} />
      {status}
    </span>
  );
}
