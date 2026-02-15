import { useEffect, useRef, useState, useCallback, useMemo } from "react";
import { User, Bot, Wrench } from "lucide-react";
import { events } from "@/lib/tauri";
import { Spinner } from "@/components/Spinner/Spinner";
import styles from "./AIConversationView.module.css";

interface AIConversationViewProps {
  sessionId: string;
}

type MessageRole = "user" | "assistant" | "tool" | "system";

interface Message {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: Date;
}

const ANSI_REGEX = /\x1b\[[0-9;]*[a-zA-Z]|\x1b\][^\x07]*\x07|\x1b[()][A-B0-2]|\x1b\[[\?]?[0-9;]*[hlm]/g;

const PROMPT_PATTERNS = [
  /^>\s/,
  /^❯\s/,
  /^\$\s/,
  /^human[:\s]/i,
  /^user[:\s]/i,
  /^Human[:\s]/,
];

const TOOL_PATTERNS = [
  /^(Read|Edit|Write|Bash|Glob|Grep|WebFetch|WebSearch)\(/,
  /^\s*(Read|Edit|Write|Bash|Glob|Grep)\s*\(/,
  /^(Reading|Editing|Writing|Running|Searching)\s/,
  /^[─━╌┄]+$/,
  /^\s*\/(Users|home|tmp|var)\//,
];

function stripAnsi(text: string): string {
  return text.replace(ANSI_REGEX, "");
}

function classifyBlock(text: string): MessageRole {
  const trimmed = text.trim();
  if (!trimmed) return "system";

  const lines = trimmed.split("\n");
  const firstLine = lines[0].trim();

  for (const pattern of PROMPT_PATTERNS) {
    if (pattern.test(firstLine)) return "user";
  }

  for (const pattern of TOOL_PATTERNS) {
    if (pattern.test(firstLine)) return "tool";
  }

  if (lines.length <= 2 && (firstLine.endsWith("?") || firstLine.length < 80)) {
    const looksLikePrompt = !firstLine.includes("  ") && /^[A-Z]/.test(firstLine);
    if (looksLikePrompt && firstLine.endsWith("?")) return "user";
  }

  return "assistant";
}

function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 5) return "just now";
  if (diffSec < 60) return `${diffSec}s ago`;

  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;

  const hours = date.getHours().toString().padStart(2, "0");
  const mins = date.getMinutes().toString().padStart(2, "0");
  const secs = date.getSeconds().toString().padStart(2, "0");
  return `${hours}:${mins}:${secs}`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderMarkdownLite(text: string): string {
  let html = escapeHtml(text);

  html = html.replace(/^### (.+)$/gm, '<strong style="font-size:14px">$1</strong>');
  html = html.replace(/^## (.+)$/gm, '<strong style="font-size:15px">$1</strong>');
  html = html.replace(/^# (.+)$/gm, '<strong style="font-size:16px">$1</strong>');

  const codeBlockParts = html.split("```");
  if (codeBlockParts.length >= 3) {
    html = "";
    for (let i = 0; i < codeBlockParts.length; i++) {
      if (i % 2 === 0) {
        html += codeBlockParts[i];
      } else {
        const codeContent = codeBlockParts[i].replace(/^[a-z]*\n/, "");
        html += `<pre class="${styles.codeBlock}"><button class="${styles.copyBtn}" data-copy-code aria-label="Copy code">\u00A0</button>${codeContent}</pre>`;
      }
    }
  }

  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/`([^`]+)`/g, `<code style="background:var(--color-bg-tertiary);padding:1px 4px;border-radius:3px;font-family:var(--font-mono);font-size:12px">$1</code>`);

  const allowedTags = /^<(\/?(strong|code|pre|button|br))\b[^>]*>$/i;
  html = html.replace(/<[^>]+>/g, (tag) => {
    if (allowedTags.test(tag)) return tag;
    return escapeHtml(tag);
  });

  return html;
}

const ROLE_LABELS: Record<MessageRole, string> = {
  user: "You",
  assistant: "Assistant",
  tool: "Tool",
  system: "System",
};

function RoleIcon({ role }: { role: MessageRole }) {
  switch (role) {
    case "user": return <User size={16} />;
    case "assistant": return <Bot size={16} />;
    case "tool": return <Wrench size={16} />;
    default: return null;
  }
}

let idCounter = 0;

export default function AIConversationView({ sessionId }: AIConversationViewProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [collapsedTools, setCollapsedTools] = useState<Set<string>>(new Set());
  const bufferRef = useRef("");
  const containerRef = useRef<HTMLDivElement>(null);
  const parseTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const parseBuffer = useCallback(() => {
    const raw = bufferRef.current;
    if (!raw) return;

    const cleaned = stripAnsi(raw);
    const blocks = cleaned.split(/\n{2,}/);
    const newMessages: Message[] = [];

    for (const block of blocks) {
      const trimmed = block.trim();
      if (!trimmed) continue;

      const role = classifyBlock(trimmed);
      idCounter += 1;
      newMessages.push({
        id: `msg-${idCounter}`,
        role,
        content: trimmed,
        timestamp: new Date(),
      });
    }

    if (newMessages.length > 0) {
      setMessages((prev) => {
        if (prev.length > 0 && newMessages.length > 0) {
          const last = prev[prev.length - 1];
          const first = newMessages[0];
          if (last.role === first.role && last.role !== "user") {
            const merged = [...prev];
            merged[merged.length - 1] = {
              ...last,
              content: last.content + "\n\n" + first.content,
            };
            return [...merged, ...newMessages.slice(1)];
          }
        }
        return [...prev, ...newMessages];
      });
    }

    bufferRef.current = "";
  }, []);

  useEffect(() => {
    const outputUnlisten = events.onPtyOutput(sessionId, (payload) => {
      const raw = atob(payload.data);
      const bytes = new Uint8Array(raw.length);
      for (let i = 0; i < raw.length; i++) {
        bytes[i] = raw.charCodeAt(i);
      }
      const decoded = new TextDecoder().decode(bytes);
      bufferRef.current += decoded;

      if (parseTimeoutRef.current) clearTimeout(parseTimeoutRef.current);
      parseTimeoutRef.current = setTimeout(parseBuffer, 150);
    });

    const exitUnlisten = events.onPtyExit((payload) => {
      if (payload.session_id === sessionId) {
        if (parseTimeoutRef.current) clearTimeout(parseTimeoutRef.current);
        parseBuffer();

        idCounter += 1;
        const code = payload.exit_code !== null ? ` with code ${payload.exit_code}` : "";
        setMessages((prev) => [
          ...prev,
          {
            id: `msg-${idCounter}`,
            role: "system",
            content: `Process exited${code}`,
            timestamp: new Date(),
          },
        ]);
      }
    });

    return () => {
      outputUnlisten.then((fn) => fn());
      exitUnlisten.then((fn) => fn());
      if (parseTimeoutRef.current) clearTimeout(parseTimeoutRef.current);
    };
  }, [sessionId, parseBuffer]);

  useEffect(() => {
    const el = containerRef.current;
    if (el) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const handleCopyClick = (e: MouseEvent) => {
      const btn = (e.target as HTMLElement).closest("[data-copy-code]");
      if (!btn) return;
      e.stopPropagation();
      const pre = btn.closest("pre");
      if (!pre) return;
      const text = pre.textContent?.replace(/^\s/, "") || "";
      navigator.clipboard.writeText(text).then(() => {
        btn.textContent = "\u2713";
        setTimeout(() => { btn.textContent = "\u00A0"; }, 2000);
      });
    };

    el.addEventListener("click", handleCopyClick);
    return () => el.removeEventListener("click", handleCopyClick);
  }, []);

  const toggleToolCollapse = useCallback((id: string) => {
    setCollapsedTools((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const renderedMessages = useMemo(() => {
    return messages.map((msg) => {
      const roleClass = styles[msg.role] || "";
      const isCollapsed = msg.role === "tool" && collapsedTools.has(msg.id);

      const messageClasses = [
        styles.message,
        roleClass,
        msg.role === "tool" && isCollapsed ? styles.toolCollapsed : "",
      ]
        .filter(Boolean)
        .join(" ");

      return (
        <div
          key={msg.id}
          className={messageClasses}
          onClick={msg.role === "tool" ? () => toggleToolCollapse(msg.id) : undefined}
        >
          {msg.role !== "system" && (
            <div className={styles.roleLabel}>
              <RoleIcon role={msg.role} />
              {ROLE_LABELS[msg.role]}
            </div>
          )}
          <div
            dangerouslySetInnerHTML={{
              __html: renderMarkdownLite(msg.content),
            }}
          />
          <div className={styles.timestamp}>{formatRelativeTime(msg.timestamp)}</div>
        </div>
      );
    });
  }, [messages, collapsedTools, toggleToolCollapse]);

  if (messages.length === 0) {
    return (
      <div className={styles.container}>
        <div className={styles.empty}><Spinner size="sm" /> Waiting for output...</div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className={styles.container}>
      {renderedMessages}
    </div>
  );
}
