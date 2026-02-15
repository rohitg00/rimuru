import { useState, useEffect, useCallback } from "react";
import { BookOpen, Play, FolderOpen } from "lucide-react";
import { commands } from "@/lib/tauri";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import PlaybookRunner from "@/components/PlaybookRunner/PlaybookRunner";
import styles from "./Playbooks.module.css";

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

export default function Playbooks() {
  const [playbooks, setPlaybooks] = useState<Playbook[]>([]);
  const [activePlaybook, setActivePlaybook] = useState<Playbook | null>(null);

  const fetchPlaybooks = useCallback(async () => {
    try {
      const list = await commands.listPlaybooks();
      setPlaybooks(list);
    } catch {
      // backend may not be ready
    }
  }, []);

  useEffect(() => {
    fetchPlaybooks();
  }, [fetchPlaybooks]);

  if (activePlaybook) {
    return (
      <PlaybookRunner
        playbook={activePlaybook}
        onClose={() => setActivePlaybook(null)}
      />
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Playbooks</h1>
        <div className={styles.headerActions}>
          <button className="btn btn-secondary" onClick={fetchPlaybooks}>
            <FolderOpen size={14} />
            Refresh
          </button>
        </div>
      </div>

      {playbooks.length === 0 ? (
        <EmptyState
          icon={BookOpen}
          title="No playbooks found"
          description="Create a playbook to automate workflows"
        />
      ) : (
        <div className={styles.grid}>
          {playbooks.map((pb) => (
            <div key={pb.id} className={styles.card}>
              <div className={styles.cardHeader}>
                <h3 className={styles.cardName}>{pb.name}</h3>
                <span className={styles.stepCount}>
                  {pb.steps.length} step{pb.steps.length !== 1 ? "s" : ""}
                </span>
              </div>
              {pb.description && (
                <p className={styles.cardDesc}>{pb.description}</p>
              )}
              <div className={styles.stepList}>
                {pb.steps.map((step, i) => (
                  <div key={i} className={styles.stepItem}>
                    <span className={styles.stepNum}>{i + 1}</span>
                    <span className={styles.stepName}>{step.name}</span>
                    <span
                      className={`${styles.gateBadge} ${
                        step.gate === "approval" ? styles.gateApproval : ""
                      }`}
                    >
                      {step.gate}
                    </span>
                  </div>
                ))}
              </div>
              <div className={styles.cardFooter}>
                <span className={styles.filePath}>{pb.file_path}</span>
                <button
                  className={styles.runBtn}
                  onClick={() => setActivePlaybook(pb)}
                >
                  <Play size={12} />
                  Run
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
