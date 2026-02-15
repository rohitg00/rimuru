import { useState, useEffect } from "react";
import { X, ChevronLeft, ChevronRight, Play } from "lucide-react";
import type { AgentWithStatus } from "@/lib/tauri";
import { commands } from "@/lib/tauri";
import type { LaunchRequest } from "@/types/pty";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import { useToast } from "@/components/Toast/ToastProvider";
import WizardStep from "./WizardStep";
import ProviderCard from "./ProviderCard";
import styles from "./LaunchWizard.module.css";

interface LaunchWizardProps {
  isOpen: boolean;
  onClose: () => void;
  onLaunch: (request: LaunchRequest) => void;
  agents: AgentWithStatus[];
  preSelectedAgent?: string;
}

const PROVIDERS = [
  { name: "Claude Code", icon: "\u27C1", type: "claude_code" },
  { name: "Codex", icon: "\u25CE", type: "codex" },
  { name: "OpenCode", icon: "\u25C7", type: "open_code" },
  { name: "Goose", icon: "\u2B21", type: "goose" },
  { name: "Gemini CLI", icon: "\u2726", type: "gemini_cli", comingSoon: true },
  { name: "Qwen3", icon: "Q", type: "qwen3", comingSoon: true },
];

const TOTAL_STEPS = 5;

export default function LaunchWizard({
  isOpen,
  onClose,
  onLaunch,
  agents,
  preSelectedAgent,
}: LaunchWizardProps) {
  const { toast } = useToast();
  const [currentStep, setCurrentStep] = useState(1);
  const [agentType, setAgentType] = useState(preSelectedAgent ?? "");
  const [workingDir, setWorkingDir] = useState("");
  const [projectDescription, setProjectDescription] = useState("");
  const [yoloMode, setYoloMode] = useState(false);
  const [useWorktree, setUseWorktree] = useState(false);
  const [branchName, setBranchName] = useState("");
  const [model, setModel] = useState("auto");
  const [maxTokens, setMaxTokens] = useState(8192);
  const [initialPrompt, setInitialPrompt] = useState("");
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState("");
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);

  useEffect(() => {
    if (preSelectedAgent) setAgentType(preSelectedAgent);
  }, [preSelectedAgent]);

  useEffect(() => {
    if (isOpen) {
      setCurrentStep(preSelectedAgent ? 2 : 1);
      setError("");
    }
  }, [isOpen, preSelectedAgent]);

  if (!shouldRender) return null;

  const installedTypes = new Set(agents.map((a) => a.agent.agent_type));

  const getProviderStatus = (type: string, comingSoon?: boolean): "installed" | "not-installed" | "coming-soon" => {
    if (comingSoon) return "coming-soon";
    return installedTypes.has(type) ? "installed" : "not-installed";
  };

  const defaultBranch = () => {
    const agent = agentType.replace(/_/g, "-");
    return `agent/${agent}-${Date.now()}`;
  };

  const confidenceScore = projectDescription.length > 0
    ? Math.min(100, Math.round((projectDescription.length / 200) * 100))
    : 0;

  const canGoNext = () => {
    if (currentStep === 1) return !!agentType;
    if (currentStep === 2) return !!workingDir;
    return true;
  };

  const handleNext = () => {
    if (currentStep < TOTAL_STEPS && canGoNext()) {
      setCurrentStep(currentStep + 1);
      setError("");
    }
  };

  const handleBack = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1);
      setError("");
    }
  };

  const handleLaunch = async () => {
    setError("");
    setLaunching(true);

    try {
      let effectiveDir = workingDir;

      if (useWorktree && workingDir) {
        const branch = branchName || defaultBranch();
        effectiveDir = await commands.createWorktree(workingDir, branch);
      }

      onLaunch({
        agent_type: agentType,
        working_dir: effectiveDir,
        initial_prompt: initialPrompt || undefined,
      });
      toast({ type: "success", title: "Agent launched", description: `${agentType} session started` });
      onClose();
    } catch (err) {
      setError(String(err));
      toast({ type: "error", title: "Launch failed", description: String(err) });
    } finally {
      setLaunching(false);
    }
  };

  const providerDisplayName = PROVIDERS.find((p) => p.type === agentType)?.name ?? agentType;

  const renderStep = () => {
    switch (currentStep) {
      case 1:
        return (
          <WizardStep step={1} title="Select Provider" subtitle="Choose the AI agent to launch">
            <div className={styles.providerGrid}>
              {PROVIDERS.map((p) => (
                <ProviderCard
                  key={p.type}
                  name={p.name}
                  icon={p.icon}
                  status={getProviderStatus(p.type, p.comingSoon)}
                  selected={agentType === p.type}
                  onClick={() => setAgentType(p.type)}
                />
              ))}
            </div>
          </WizardStep>
        );

      case 2:
        return (
          <WizardStep step={2} title="Project Directory" subtitle="Set the working directory for this session">
            <div className={styles.field}>
              <label className={styles.label}>Working Directory</label>
              <input
                type="text"
                className={styles.input}
                value={workingDir}
                onChange={(e) => setWorkingDir(e.target.value)}
                placeholder="/path/to/project"
              />
            </div>
            <label className={styles.toggle}>
              <input
                type="checkbox"
                checked={yoloMode}
                onChange={(e) => setYoloMode(e.target.checked)}
                className={styles.checkbox}
              />
              <div>
                <span className={styles.toggleLabel}>YOLO Mode</span>
                <span className={styles.hint}>Auto-approve all tool calls without confirmation</span>
              </div>
            </label>
          </WizardStep>
        );

      case 3:
        return (
          <WizardStep step={3} title="Project Discovery" subtitle="Describe your project for better context">
            <div className={styles.field}>
              <label className={styles.label}>Project Description</label>
              <textarea
                className={styles.textarea}
                value={projectDescription}
                onChange={(e) => setProjectDescription(e.target.value)}
                placeholder="Describe what this project does, its tech stack, and what you want to work on..."
                rows={4}
              />
            </div>
            <div className={styles.field}>
              <label className={styles.label}>Context Confidence</label>
              <div className={styles.gauge}>
                <div className={styles.gaugeFill} style={{ width: `${confidenceScore}%` }} />
              </div>
              <span className={styles.hint}>{confidenceScore}% - {confidenceScore < 30 ? "Minimal context" : confidenceScore < 70 ? "Good context" : "Rich context"}</span>
            </div>
          </WizardStep>
        );

      case 4:
        return (
          <WizardStep step={4} title="Configuration" subtitle="Fine-tune session settings">
            <label className={styles.toggle}>
              <input
                type="checkbox"
                checked={useWorktree}
                onChange={(e) => setUseWorktree(e.target.checked)}
                className={styles.checkbox}
              />
              <span>Launch in git worktree</span>
            </label>
            {useWorktree && (
              <div className={styles.field}>
                <label className={styles.label}>Branch Name</label>
                <input
                  type="text"
                  className={styles.input}
                  value={branchName}
                  onChange={(e) => setBranchName(e.target.value)}
                  placeholder={defaultBranch()}
                />
                <span className={styles.hint}>Leave empty for auto-generated name</span>
              </div>
            )}
            <div className={styles.field}>
              <label className={styles.label}>Model</label>
              <select
                className={styles.select}
                value={model}
                onChange={(e) => setModel(e.target.value)}
              >
                <option value="auto">Auto</option>
                <option value="claude-4-sonnet">Claude 4 Sonnet</option>
                <option value="claude-4-opus">Claude 4 Opus</option>
              </select>
            </div>
            <div className={styles.field}>
              <label className={styles.label}>Max Tokens</label>
              <select
                className={styles.select}
                value={maxTokens}
                onChange={(e) => setMaxTokens(Number(e.target.value))}
              >
                <option value={4096}>4,096</option>
                <option value={8192}>8,192</option>
                <option value={16384}>16,384</option>
                <option value={32768}>32,768</option>
              </select>
            </div>
          </WizardStep>
        );

      case 5:
        return (
          <WizardStep step={5} title="Review & Launch" subtitle="Confirm your session settings">
            <div className={styles.summary}>
              <div className={styles.summaryRow}>
                <span className={styles.summaryLabel}>Provider</span>
                <span className={styles.summaryValue}>{providerDisplayName}</span>
              </div>
              <div className={styles.summaryRow}>
                <span className={styles.summaryLabel}>Directory</span>
                <span className={styles.summaryValue}>{workingDir}</span>
              </div>
              <div className={styles.summaryRow}>
                <span className={styles.summaryLabel}>YOLO Mode</span>
                <span className={styles.summaryValue}>{yoloMode ? "Enabled" : "Disabled"}</span>
              </div>
              <div className={styles.summaryRow}>
                <span className={styles.summaryLabel}>Model</span>
                <span className={styles.summaryValue}>{model === "auto" ? "Auto" : model}</span>
              </div>
              <div className={styles.summaryRow}>
                <span className={styles.summaryLabel}>Max Tokens</span>
                <span className={styles.summaryValue}>{maxTokens.toLocaleString()}</span>
              </div>
              {useWorktree && (
                <div className={styles.summaryRow}>
                  <span className={styles.summaryLabel}>Worktree Branch</span>
                  <span className={styles.summaryValue}>{branchName || defaultBranch()}</span>
                </div>
              )}
            </div>
            <div className={styles.field}>
              <label className={styles.label}>Initial Prompt</label>
              <textarea
                className={styles.textarea}
                value={initialPrompt}
                onChange={(e) => setInitialPrompt(e.target.value)}
                placeholder="Enter a prompt to send after launch..."
                rows={3}
              />
            </div>
            {error && <div className={styles.error}>{error}</div>}
          </WizardStep>
        );

      default:
        return null;
    }
  };

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <h2 className={styles.title}>Launch Agent Session</h2>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog">
            <X size={18} />
          </button>
        </div>

        <div className={styles.dots}>
          {Array.from({ length: TOTAL_STEPS }, (_, i) => i + 1).map((s) => (
            <span
              key={s}
              className={`${styles.dot} ${s === currentStep ? styles.dotActive : ""} ${s < currentStep ? styles.dotCompleted : ""}`}
            />
          ))}
        </div>

        <div className={styles.body}>
          {renderStep()}
        </div>

        <div className={styles.footer}>
          <button
            className={styles.footerBtn}
            onClick={handleBack}
            disabled={currentStep === 1}
          >
            <ChevronLeft size={16} />
            Back
          </button>
          {currentStep < TOTAL_STEPS ? (
            <button
              className={`${styles.footerBtn} ${styles.footerBtnPrimary}`}
              onClick={handleNext}
              disabled={!canGoNext()}
            >
              Next
              <ChevronRight size={16} />
            </button>
          ) : (
            <button
              className={`${styles.footerBtn} ${styles.footerBtnPrimary}`}
              onClick={handleLaunch}
              disabled={launching}
            >
              <Play size={16} />
              {launching ? "Launching..." : "Launch"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
