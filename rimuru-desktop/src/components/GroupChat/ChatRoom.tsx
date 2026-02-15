import { useState, useRef, useEffect, useCallback } from "react";
import { Send } from "lucide-react";
import type { ChatRoom as ChatRoomType, ChatMessage } from "@/lib/tauri";
import { commands } from "@/lib/tauri";
import styles from "./ChatRoom.module.css";

const AGENT_COLORS = [
  "#7c3aed",
  "#059669",
  "#d97706",
  "#dc2626",
  "#2563eb",
  "#db2777",
  "#0891b2",
  "#65a30d",
];

interface ChatRoomProps {
  room: ChatRoomType;
}

export default function ChatRoom({ room }: ChatRoomProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [text, setText] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const fetchMessages = useCallback(async () => {
    try {
      const msgs = await commands.getChatMessages(room.id);
      setMessages(msgs);
    } catch {
      // backend may not be ready
    }
  }, [room.id]);

  useEffect(() => {
    fetchMessages();
    const interval = setInterval(fetchMessages, 1000);
    return () => clearInterval(interval);
  }, [fetchMessages]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length]);

  const agentColorMap = new Map<string, string>();
  room.agents.forEach((agent, i) => {
    agentColorMap.set(agent.name, AGENT_COLORS[i % AGENT_COLORS.length]);
  });

  const handleSend = useCallback(async () => {
    const trimmed = text.trim();
    if (!trimmed) return;
    setText("");
    try {
      await commands.sendChatMessage(room.id, trimmed);
      await fetchMessages();
    } catch (err) {
      console.error("Failed to send message:", err);
    }
  }, [text, room.id, fetchMessages]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const autoResize = useCallback(() => {
    const ta = textareaRef.current;
    if (!ta) return;
    ta.style.height = "auto";
    ta.style.height = `${Math.min(ta.scrollHeight, 120)}px`;
  }, []);

  const formatTime = (timestamp: string) => {
    const d = new Date(timestamp);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  return (
    <div className={styles.container}>
      <div className={styles.agentBar}>
        {room.agents.map((agent, i) => (
          <div key={agent.name} className={styles.agentChip}>
            <span
              className={styles.agentDot}
              style={{ backgroundColor: AGENT_COLORS[i % AGENT_COLORS.length] }}
            />
            {agent.name}
            {agent.role !== agent.name && ` (${agent.role})`}
          </div>
        ))}
      </div>

      <div className={styles.messages}>
        {messages.map((msg) => {
          const isUser = msg.message_type === "user";
          const isSystem = msg.message_type === "system";
          const color = agentColorMap.get(msg.sender) || "#6b7280";

          return (
            <div
              key={msg.id}
              className={`${styles.messageRow} ${
                isUser ? styles.messageRowUser
                : isSystem ? styles.messageRowSystem
                : styles.messageRowAgent
              }`}
            >
              {!isSystem && (
                <div
                  className={styles.avatar}
                  style={{ backgroundColor: isUser ? "var(--color-accent)" : color }}
                >
                  {msg.sender.charAt(0).toUpperCase()}
                </div>
              )}
              <div>
                {!isUser && !isSystem && (
                  <div className={styles.senderName} style={{ color }}>
                    {msg.sender}
                  </div>
                )}
                <div
                  className={`${styles.bubble} ${
                    isUser ? styles.bubbleUser
                    : isSystem ? styles.bubbleSystem
                    : styles.bubbleAgent
                  }`}
                >
                  {msg.content}
                </div>
                {!isSystem && (
                  <div className={styles.messageTime}>{formatTime(msg.timestamp)}</div>
                )}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      <div className={styles.inputArea}>
        <textarea
          ref={textareaRef}
          className={styles.inputField}
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            autoResize();
          }}
          onKeyDown={handleKeyDown}
          placeholder="Send a message to all agents..."
          rows={1}
        />
        <button
          className={styles.sendBtn}
          onClick={handleSend}
          disabled={!text.trim()}
        >
          <Send size={16} />
        </button>
      </div>
    </div>
  );
}
