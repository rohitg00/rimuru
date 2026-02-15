import { useState, useCallback } from "react";
import { X } from "lucide-react";
import styles from "./CreateRoomModal.module.css";

const AGENT_TYPES = [
  { value: "claude_code", label: "Claude Code" },
  { value: "codex", label: "Codex" },
  { value: "goose", label: "Goose" },
  { value: "opencode", label: "OpenCode" },
  { value: "cursor", label: "Cursor" },
  { value: "copilot", label: "Copilot" },
];

interface AgentEntry {
  agent_type: string;
  name: string;
  role: string;
  selected: boolean;
}

interface CreateRoomModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (name: string, agents: Array<{ agent_type: string; name: string; role: string }>) => void;
}

export default function CreateRoomModal({ isOpen, onClose, onCreate }: CreateRoomModalProps) {
  const [roomName, setRoomName] = useState("");
  const [agents, setAgents] = useState<AgentEntry[]>(
    AGENT_TYPES.map((a) => ({
      agent_type: a.value,
      name: a.label,
      role: "",
      selected: false,
    }))
  );

  const toggleAgent = useCallback((index: number) => {
    setAgents((prev) =>
      prev.map((a, i) => (i === index ? { ...a, selected: !a.selected } : a))
    );
  }, []);

  const updateRole = useCallback((index: number, role: string) => {
    setAgents((prev) =>
      prev.map((a, i) => (i === index ? { ...a, role } : a))
    );
  }, []);

  const handleCreate = useCallback(() => {
    const selected = agents
      .filter((a) => a.selected)
      .map((a) => ({
        agent_type: a.agent_type,
        name: a.name,
        role: a.role || a.name,
      }));
    if (!roomName.trim() || selected.length === 0) return;
    onCreate(roomName.trim(), selected);
    setRoomName("");
    setAgents((prev) => prev.map((a) => ({ ...a, selected: false, role: "" })));
    onClose();
  }, [roomName, agents, onCreate, onClose]);

  if (!isOpen) return null;

  const selectedCount = agents.filter((a) => a.selected).length;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <h2>New Chat Room</h2>
          <button className={styles.closeBtn} onClick={onClose}>
            <X size={16} />
          </button>
        </div>

        <div className={styles.body}>
          <div className={styles.field}>
            <label>Room Name</label>
            <input
              value={roomName}
              onChange={(e) => setRoomName(e.target.value)}
              placeholder="e.g. Feature Sprint"
              autoFocus
            />
          </div>

          <div className={styles.field}>
            <label>Select Agents</label>
            <div className={styles.agentGrid}>
              {agents.map((agent, i) => (
                <div
                  key={agent.agent_type}
                  className={`${styles.agentOption} ${agent.selected ? styles.agentSelected : ""}`}
                  onClick={() => toggleAgent(i)}
                >
                  <input
                    type="checkbox"
                    checked={agent.selected}
                    onChange={() => toggleAgent(i)}
                  />
                  <div>
                    <div className={styles.agentLabel}>{agent.name}</div>
                    {agent.selected && (
                      <input
                        className={styles.roleInput}
                        value={agent.role}
                        onChange={(e) => {
                          e.stopPropagation();
                          updateRole(i, e.target.value);
                        }}
                        onClick={(e) => e.stopPropagation()}
                        placeholder="Role (optional)"
                      />
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className={styles.footer}>
          <button className="btn" onClick={onClose}>Cancel</button>
          <button
            className="btn btn-primary"
            onClick={handleCreate}
            disabled={!roomName.trim() || selectedCount === 0}
          >
            Create Room
          </button>
        </div>
      </div>
    </div>
  );
}
