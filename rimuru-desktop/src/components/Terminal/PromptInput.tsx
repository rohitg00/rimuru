import { useState, useRef, useEffect, useCallback } from "react";
import { Send, ToggleRight } from "lucide-react";
import { commands } from "@/lib/tauri";
import styles from "./PromptInput.module.css";

interface PromptInputProps {
  sessionId: string;
  disabled: boolean;
  onModeChange: (mode: "smart" | "raw") => void;
  mode: "smart" | "raw";
}

export default function PromptInput({
  sessionId,
  disabled,
  onModeChange,
}: PromptInputProps) {
  const [text, setText] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const autoResize = useCallback(() => {
    const ta = textareaRef.current;
    if (!ta) return;
    ta.style.height = "auto";
    ta.style.height = `${Math.min(ta.scrollHeight, 120)}px`;
  }, []);

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed || disabled) return;

    const payload = trimmed + "\n";
    const base64 = btoa(payload);
    commands.writeToSession(sessionId, base64);
    setText("");

    requestAnimationFrame(() => {
      if (textareaRef.current) {
        textareaRef.current.style.height = "auto";
      }
    });
  }, [text, disabled, sessionId]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const toggleMode = useCallback(() => {
    onModeChange("raw");
  }, [onModeChange]);

  return (
    <div className={styles.container}>
      <div className={styles.inputRow}>
        <textarea
          ref={textareaRef}
          className={styles.textarea}
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            autoResize();
          }}
          onKeyDown={handleKeyDown}
          placeholder={disabled ? "Session not running..." : "Type a message..."}
          disabled={disabled}
          rows={1}
        />
        <button
          className={styles.sendBtn}
          onClick={handleSend}
          disabled={disabled || !text.trim()}
        >
          <Send size={16} />
        </button>
      </div>
      <div className={styles.toolbar}>
        <button className={styles.modeToggle} onClick={toggleMode}>
          <ToggleRight size={12} style={{ marginRight: 4, verticalAlign: "middle" }} />
          Smart
        </button>
      </div>
    </div>
  );
}
