import { useState } from "react";
import { X, ArrowRight, AlertCircle, CheckCircle, AlertTriangle } from "lucide-react";
import { useTranslateSkill } from "@/hooks/useSkills";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import { Skill, InstalledSkill, SKILLKIT_AGENTS, TranslationResult } from "@/lib/tauri";
import styles from "./TranslateModal.module.css";
import clsx from "clsx";

interface TranslateModalProps {
  isOpen?: boolean;
  skill: Skill | InstalledSkill;
  onClose: () => void;
  onSuccess?: () => void;
}

function isInstalledSkill(skill: Skill | InstalledSkill): skill is InstalledSkill {
  return "installed_at" in skill;
}

export default function TranslateModal({
  isOpen = true,
  skill,
  onClose,
  onSuccess,
}: TranslateModalProps) {
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);
  const installed = isInstalledSkill(skill);
  const availableAgents = installed ? skill.agents : [];

  const [fromAgent, setFromAgent] = useState<string>(
    availableAgents[0] ?? SKILLKIT_AGENTS[0].value
  );
  const [toAgent, setToAgent] = useState<string>("");
  const [result, setResult] = useState<TranslationResult | null>(null);
  const translateMutation = useTranslateSkill();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!fromAgent || !toAgent || fromAgent === toAgent) return;

    try {
      const translationResult = await translateMutation.mutateAsync({
        skill_id: skill.id,
        from_agent: fromAgent,
        to_agent: toAgent,
      });
      setResult(translationResult);
      onSuccess?.();
    } catch (error) {
      console.error("Failed to translate skill:", error);
    }
  };

  const getAgentLabel = (value: string): string => {
    const agent = SKILLKIT_AGENTS.find((a) => a.value === value);
    return agent?.label ?? value;
  };

  const getAgentIcon = (value: string): string => {
    const agent = SKILLKIT_AGENTS.find((a) => a.value === value);
    return agent?.icon ?? "●";
  };

  if (!shouldRender) return null;

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <div>
            <h2 className={styles.title}>Translate Skill</h2>
            <p className={styles.skillName}>{skill.name}</p>
          </div>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog">
            <X size={20} />
          </button>
        </div>

        {result ? (
          <div className={styles.content}>
            <div className={clsx(styles.resultBox, result.success ? styles.success : styles.error)}>
              {result.success ? (
                <CheckCircle size={24} className={styles.resultIcon} />
              ) : (
                <AlertCircle size={24} className={styles.resultIcon} />
              )}
              <div className={styles.resultText}>
                <h3>{result.success ? "Translation Successful" : "Translation Failed"}</h3>
                <p>
                  {result.success
                    ? `Skill translated from ${getAgentLabel(result.original_agent)} to ${getAgentLabel(result.target_agent)}`
                    : "An error occurred during translation."}
                </p>
                {result.success && result.output_path && (
                  <code className={styles.outputPath}>{result.output_path}</code>
                )}
              </div>
            </div>

            {result.warnings.length > 0 && (
              <div className={styles.warnings}>
                <div className={styles.warningHeader}>
                  <AlertTriangle size={16} />
                  <span>Warnings</span>
                </div>
                <ul className={styles.warningList}>
                  {result.warnings.map((warning, i) => (
                    <li key={i}>{warning}</li>
                  ))}
                </ul>
              </div>
            )}

            <div className={styles.footer}>
              <button className="btn btn-primary" onClick={onClose}>
                Done
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit}>
            <div className={styles.content}>
              <p className={styles.description}>
                Translate this skill's configuration from one agent format to another.
              </p>

              <div className={styles.translationFlow}>
                <div className={styles.agentSelect}>
                  <label className={styles.label}>From Agent</label>
                  <select
                    value={fromAgent}
                    onChange={(e) => setFromAgent(e.target.value)}
                    className={styles.select}
                  >
                    {(installed ? availableAgents : SKILLKIT_AGENTS.map((a) => a.value)).map(
                      (agentValue) => (
                        <option key={agentValue} value={agentValue}>
                          {getAgentIcon(agentValue)} {getAgentLabel(agentValue)}
                        </option>
                      )
                    )}
                  </select>
                </div>

                <div className={styles.arrow}>
                  <ArrowRight size={24} />
                </div>

                <div className={styles.agentSelect}>
                  <label className={styles.label}>To Agent</label>
                  <select
                    value={toAgent}
                    onChange={(e) => setToAgent(e.target.value)}
                    className={styles.select}
                  >
                    <option value="">Select target agent...</option>
                    {SKILLKIT_AGENTS.filter((a) => a.value !== fromAgent).map((agent) => (
                      <option key={agent.value} value={agent.value}>
                        {agent.icon} {agent.label}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {translateMutation.isError && (
                <div className={styles.errorBox}>
                  <AlertCircle size={16} />
                  <span>Failed to translate skill. Please try again.</span>
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
                disabled={!fromAgent || !toAgent || fromAgent === toAgent || translateMutation.isPending}
              >
                {translateMutation.isPending ? "Translating..." : "Translate"}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
