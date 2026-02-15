import { Download, Tag, User, Check, Clock } from "lucide-react";
import { Skill, InstalledSkill, SKILLKIT_AGENTS } from "@/lib/tauri";
import styles from "./SkillCard.module.css";
import clsx from "clsx";

interface SkillCardProps {
  skill: Skill | InstalledSkill;
  onClick?: () => void;
  onInstall?: () => void;
  showInstallButton?: boolean;
  confidence?: number;
  reason?: string;
}

function isInstalledSkill(skill: Skill | InstalledSkill): skill is InstalledSkill {
  return "installed_at" in skill;
}

function formatDownloads(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

function getAgentIcon(agentValue: string): string {
  const agent = SKILLKIT_AGENTS.find((a) => a.value === agentValue);
  return agent?.icon ?? "●";
}

export default function SkillCard({
  skill,
  onClick,
  onInstall,
  showInstallButton = false,
  confidence,
  reason,
}: SkillCardProps) {
  const isInstalled = isInstalledSkill(skill);

  const handleInstallClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onInstall?.();
  };

  return (
    <div className={styles.card} onClick={onClick} role="button" tabIndex={0}>
      <div className={styles.header}>
        <div className={styles.titleRow}>
          <h3 className={styles.name}>{skill.name}</h3>
          {isInstalled && (
            <span
              className={clsx(
                "badge",
                skill.enabled ? "badge-success" : "badge-warning"
              )}
            >
              {skill.enabled ? "Enabled" : "Disabled"}
            </span>
          )}
        </div>
        <p className={styles.description}>{skill.description}</p>
      </div>

      {reason && (
        <div className={styles.recommendation}>
          <div className={styles.confidenceBar}>
            <div
              className={styles.confidenceFill}
              style={{ width: `${(confidence ?? 0) * 100}%` }}
            />
          </div>
          <span className={styles.reason}>{reason}</span>
        </div>
      )}

      <div className={styles.meta}>
        <div className={styles.metaItem}>
          <User size={12} />
          <span>{skill.author}</span>
        </div>
        <div className={styles.metaItem}>
          <Download size={12} />
          <span>{formatDownloads(skill.downloads)}</span>
        </div>
        {skill.version && (
          <div className={styles.metaItem}>
            <span className={styles.version}>v{skill.version}</span>
          </div>
        )}
      </div>

      {skill.tags.length > 0 && (
        <div className={styles.tags}>
          {skill.tags.slice(0, 3).map((tag) => (
            <span key={tag} className={styles.tag}>
              <Tag size={10} />
              {tag}
            </span>
          ))}
          {skill.tags.length > 3 && (
            <span className={styles.moreTag}>+{skill.tags.length - 3}</span>
          )}
        </div>
      )}

      {isInstalled && skill.agents.length > 0 && (
        <div className={styles.agents}>
          {skill.agents.slice(0, 5).map((agent) => (
            <span key={agent} className={styles.agentIcon} title={agent}>
              {getAgentIcon(agent)}
            </span>
          ))}
          {skill.agents.length > 5 && (
            <span className={styles.moreAgents}>+{skill.agents.length - 5}</span>
          )}
        </div>
      )}

      <div className={styles.footer}>
        {isInstalled ? (
          <div className={styles.installedInfo}>
            <Check size={14} className={styles.checkIcon} />
            <Clock size={12} />
            <span>Installed {new Date(skill.installed_at).toLocaleDateString()}</span>
          </div>
        ) : (
          showInstallButton && (
            <button
              className="btn btn-primary btn-sm"
              onClick={handleInstallClick}
            >
              <Download size={14} />
              Install
            </button>
          )
        )}
      </div>
    </div>
  );
}
