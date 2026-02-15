import { useState } from "react";
import { X, Download, Check, AlertCircle } from "lucide-react";
import { useInstallSkill } from "@/hooks/useSkills";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import { useToast } from "@/components/Toast/ToastProvider";
import { Skill, SKILLKIT_AGENTS } from "@/lib/tauri";
import styles from "./SkillInstallModal.module.css";
import clsx from "clsx";

interface SkillInstallModalProps {
  isOpen?: boolean;
  skill: Skill;
  onClose: () => void;
  onSuccess?: () => void;
}

export default function SkillInstallModal({
  isOpen = true,
  skill,
  onClose,
  onSuccess,
}: SkillInstallModalProps) {
  const { toast } = useToast();
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);
  const [selectedAgents, setSelectedAgents] = useState<string[]>([]);
  const [installAll, setInstallAll] = useState(false);
  const installMutation = useInstallSkill();

  const handleToggleAgent = (agentValue: string) => {
    setSelectedAgents((prev) =>
      prev.includes(agentValue)
        ? prev.filter((a) => a !== agentValue)
        : [...prev, agentValue]
    );
    setInstallAll(false);
  };

  const handleToggleAll = () => {
    if (installAll) {
      setInstallAll(false);
      setSelectedAgents([]);
    } else {
      setInstallAll(true);
      setSelectedAgents(SKILLKIT_AGENTS.map((a) => a.value));
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (selectedAgents.length === 0) return;

    try {
      await installMutation.mutateAsync({
        skill_id: skill.id,
        agents: selectedAgents,
        install_all: installAll,
      });
      toast({ type: "success", title: "Skill installed", description: `${skill.name} installed successfully` });
      onSuccess?.();
      onClose();
    } catch (error) {
      console.error("Failed to install skill:", error);
      toast({ type: "error", title: "Install failed", description: String(error) });
    }
  };

  if (!shouldRender) return null;

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div>
            <h2 className={styles.title}>Install Skill</h2>
            <p className={styles.skillName}>{skill.name}</p>
          </div>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog">
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className={styles.content}>
            <p className={styles.description}>
              Select the agents you want to install this skill for:
            </p>

            <div className={styles.selectAllRow}>
              <label className={styles.selectAll}>
                <input
                  type="checkbox"
                  checked={installAll}
                  onChange={handleToggleAll}
                  className={styles.checkbox}
                />
                <span>Install for all agents</span>
              </label>
            </div>

            <div className={styles.agentGrid}>
              {SKILLKIT_AGENTS.map((agent) => {
                const isSelected = selectedAgents.includes(agent.value);
                return (
                  <button
                    key={agent.value}
                    type="button"
                    className={clsx(styles.agentCard, isSelected && styles.selected)}
                    onClick={() => handleToggleAgent(agent.value)}
                  >
                    <span className={styles.agentIcon}>{agent.icon}</span>
                    <span className={styles.agentLabel}>{agent.label}</span>
                    {isSelected && <Check size={14} className={styles.checkIcon} />}
                  </button>
                );
              })}
            </div>

            {installMutation.isError && (
              <div className={styles.error}>
                <AlertCircle size={16} />
                <span>Failed to install skill. Please try again.</span>
              </div>
            )}
          </div>

          <div className={styles.footer}>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={selectedAgents.length === 0 || installMutation.isPending}
            >
              <Download size={16} />
              {installMutation.isPending
                ? "Installing..."
                : `Install for ${selectedAgents.length} agent${selectedAgents.length !== 1 ? "s" : ""}`}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
