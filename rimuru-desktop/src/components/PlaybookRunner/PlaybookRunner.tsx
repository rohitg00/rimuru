import { useState, useMemo, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Play } from "lucide-react";
import StepCard, { type StepStatus } from "./StepCard";
import styles from "./PlaybookRunner.module.css";

interface PlaybookStep {
  name: string;
  prompt: string;
  agent_type: string;
  working_dir: string | null;
  gate: string;
  timeout_secs: number | null;
}

interface Playbook {
  id: string;
  name: string;
  description: string;
  steps: PlaybookStep[];
  file_path: string;
}

interface Props {
  playbook: Playbook;
  onClose: () => void;
}

export default function PlaybookRunner({ playbook, onClose }: Props) {
  const navigate = useNavigate();
  const [stepStatuses, setStepStatuses] = useState<StepStatus[]>(
    playbook.steps.map(() => "pending")
  );
  const [variables, setVariables] = useState<Record<string, string>>({});
  const [started, setStarted] = useState(false);

  const detectedVars = useMemo(() => {
    const vars = new Set<string>();
    for (const step of playbook.steps) {
      const matches = step.prompt.matchAll(/\{(\w+)\}/g);
      for (const m of matches) {
        vars.add(m[1]);
      }
    }
    return Array.from(vars);
  }, [playbook.steps]);

  const resolvePrompt = useCallback(
    (prompt: string) => {
      let resolved = prompt;
      for (const [key, val] of Object.entries(variables)) {
        resolved = resolved.split(`{${key}}`).join(val);
      }
      return resolved;
    },
    [variables]
  );

  const currentStepIndex = stepStatuses.findIndex(
    (s) => s === "pending" || s === "running" || s === "waiting_approval"
  );
  const completedCount = stepStatuses.filter(
    (s) => s === "completed" || s === "skipped"
  ).length;

  const handleStart = () => {
    setStarted(true);
    if (playbook.steps[0]?.gate === "approval") {
      setStepStatuses((prev) => {
        const next = [...prev];
        next[0] = "waiting_approval";
        return next;
      });
    }
  };

  const handleLaunchStep = (index: number) => {
    const step = playbook.steps[index];
    const resolvedPrompt = resolvePrompt(step.prompt);

    setStepStatuses((prev) => {
      const next = [...prev];
      next[index] = "running";
      return next;
    });

    const agentMap: Record<string, string> = {
      claude_code: "claude_code",
      codex: "codex",
      goose: "goose",
      opencode: "opencode",
    };

    navigate(
      `/orchestrate?agent=${agentMap[step.agent_type] ?? step.agent_type}&prompt=${encodeURIComponent(resolvedPrompt)}&cwd=${encodeURIComponent(step.working_dir ?? "~")}`
    );

    setTimeout(() => {
      setStepStatuses((prev) => {
        const next = [...prev];
        next[index] = "completed";
        const nextIdx = index + 1;
        if (nextIdx < playbook.steps.length) {
          if (playbook.steps[nextIdx].gate === "approval") {
            next[nextIdx] = "waiting_approval";
          }
        }
        return next;
      });
    }, 1000);
  };

  const handleSkipStep = (index: number) => {
    setStepStatuses((prev) => {
      const next = [...prev];
      next[index] = "skipped";
      const nextIdx = index + 1;
      if (nextIdx < playbook.steps.length) {
        if (playbook.steps[nextIdx].gate === "approval") {
          next[nextIdx] = "waiting_approval";
        }
      }
      return next;
    });
  };

  const handleApproveStep = (index: number) => {
    handleLaunchStep(index);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <button className={styles.backBtn} onClick={onClose}>
          <ArrowLeft size={16} />
        </button>
        <div className={styles.headerInfo}>
          <h2 className={styles.title}>{playbook.name}</h2>
          {playbook.description && (
            <p className={styles.description}>{playbook.description}</p>
          )}
        </div>
      </div>

      <div className={styles.statusBar}>
        <span className={styles.statusLabel}>Progress:</span>
        <span>
          {completedCount} / {playbook.steps.length} steps
        </span>
        {currentStepIndex >= 0 && (
          <>
            <span>|</span>
            <span>Current: {playbook.steps[currentStepIndex].name}</span>
          </>
        )}
      </div>

      {detectedVars.length > 0 && !started && (
        <div className={styles.variablesSection}>
          <p className={styles.variablesTitle}>Variables</p>
          {detectedVars.map((v) => (
            <div key={v} className={styles.variableRow}>
              <label className={styles.variableLabel}>{`{${v}}`}</label>
              <input
                className={styles.variableInput}
                placeholder={`Enter value for ${v}`}
                value={variables[v] ?? ""}
                onChange={(e) =>
                  setVariables((prev) => ({ ...prev, [v]: e.target.value }))
                }
              />
            </div>
          ))}
        </div>
      )}

      {!started && (
        <button className={styles.startBtn} onClick={handleStart}>
          <Play size={14} />
          Start Playbook
        </button>
      )}

      <div className={styles.pipeline}>
        {playbook.steps.map((step, i) => (
          <StepCard
            key={i}
            index={i}
            name={step.name}
            prompt={resolvePrompt(step.prompt)}
            agentType={step.agent_type}
            gate={step.gate}
            status={stepStatuses[i]}
            isLast={i === playbook.steps.length - 1}
            onLaunch={() => handleLaunchStep(i)}
            onApprove={() => handleApproveStep(i)}
            onSkip={() => handleSkipStep(i)}
            canLaunch={
              started &&
              (i === 0 ||
                stepStatuses[i - 1] === "completed" ||
                stepStatuses[i - 1] === "skipped")
            }
          />
        ))}
      </div>
    </div>
  );
}
