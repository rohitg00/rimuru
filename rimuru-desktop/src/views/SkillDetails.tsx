import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  Download,
  User,
  Tag,
  Clock,
  ExternalLink,
  Check,
  X,
  Languages,
  Power,
  Trash2,
} from "lucide-react";
import {
  useSkillDetails,
  useInstalledSkills,
  useUninstallSkill,
  useEnableSkill,
  useDisableSkill,
} from "@/hooks/useSkills";
import { SKILLKIT_AGENTS, InstalledSkill } from "@/lib/tauri";
import SkillInstallModal from "@/components/SkillInstallModal";
import TranslateModal from "@/components/TranslateModal";
import styles from "./SkillDetails.module.css";
import clsx from "clsx";

function formatDownloads(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
  return num.toString();
}

export default function SkillDetails() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: skill, isLoading } = useSkillDetails(id ?? "");
  const { data: installedSkills } = useInstalledSkills();
  const uninstallMutation = useUninstallSkill();
  const enableMutation = useEnableSkill();
  const disableMutation = useDisableSkill();

  const [showInstallModal, setShowInstallModal] = useState(false);
  const [showTranslateModal, setShowTranslateModal] = useState(false);

  const installedSkill = installedSkills?.find((s) => s.id === id) as
    | InstalledSkill
    | undefined;
  const isInstalled = !!installedSkill;
  const displaySkill = installedSkill ?? skill;

  const handleUninstall = async () => {
    if (!id) return;
    if (confirm("Are you sure you want to uninstall this skill from all agents?")) {
      await uninstallMutation.mutateAsync({ skillId: id });
      navigate("/skills");
    }
  };

  const handleToggleEnabled = async () => {
    if (!id || !installedSkill) return;
    if (installedSkill.enabled) {
      await disableMutation.mutateAsync({ skillId: id });
    } else {
      await enableMutation.mutateAsync({ skillId: id });
    }
  };

  if (isLoading) {
    return <div className={styles.loading}>Loading skill details...</div>;
  }

  if (!displaySkill) {
    return (
      <div className={styles.notFound}>
        <p>Skill not found</p>
        <button className="btn btn-primary" onClick={() => navigate("/skills")}>
          Back to Skills
        </button>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <button className={styles.backBtn} onClick={() => navigate("/skills")}>
        <ArrowLeft size={16} />
        Back to Skills
      </button>

      <div className={styles.header}>
        <div className={styles.titleSection}>
          <h1 className={styles.name}>{displaySkill.name}</h1>
          {isInstalled && (
            <span
              className={clsx(
                "badge",
                installedSkill.enabled ? "badge-success" : "badge-warning"
              )}
            >
              {installedSkill.enabled ? "Enabled" : "Disabled"}
            </span>
          )}
        </div>
        <div className={styles.actions}>
          {isInstalled ? (
            <>
              <button
                className={clsx("btn", installedSkill.enabled ? "btn-warning" : "btn-success")}
                onClick={handleToggleEnabled}
                disabled={enableMutation.isPending || disableMutation.isPending}
              >
                <Power size={16} />
                {installedSkill.enabled ? "Disable" : "Enable"}
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => setShowTranslateModal(true)}
              >
                <Languages size={16} />
                Translate
              </button>
              <button
                className="btn btn-danger"
                onClick={handleUninstall}
                disabled={uninstallMutation.isPending}
              >
                <Trash2 size={16} />
                Uninstall
              </button>
            </>
          ) : (
            <button
              className="btn btn-primary"
              onClick={() => setShowInstallModal(true)}
            >
              <Download size={16} />
              Install
            </button>
          )}
        </div>
      </div>

      <p className={styles.description}>{displaySkill.description}</p>

      <div className={styles.grid}>
        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Details</h3>
          <div className={styles.detailRow}>
            <User size={16} />
            <span className={styles.detailLabel}>Author</span>
            <span className={styles.detailValue}>{displaySkill.author}</span>
          </div>
          <div className={styles.detailRow}>
            <Tag size={16} />
            <span className={styles.detailLabel}>Version</span>
            <code className={styles.versionBadge}>v{displaySkill.version}</code>
          </div>
          <div className={styles.detailRow}>
            <Download size={16} />
            <span className={styles.detailLabel}>Downloads</span>
            <span className={styles.detailValue}>
              {formatDownloads(displaySkill.downloads)}
            </span>
          </div>
          <div className={styles.detailRow}>
            <Clock size={16} />
            <span className={styles.detailLabel}>Updated</span>
            <span className={styles.detailValue}>
              {new Date(displaySkill.updated_at).toLocaleDateString()}
            </span>
          </div>
          {displaySkill.source && (
            <div className={styles.detailRow}>
              <ExternalLink size={16} />
              <span className={styles.detailLabel}>Source</span>
              <a
                href={displaySkill.source}
                target="_blank"
                rel="noopener noreferrer"
                className={styles.sourceLink}
              >
                View on GitHub
              </a>
            </div>
          )}
        </div>

        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Tags</h3>
          {displaySkill.tags.length > 0 ? (
            <div className={styles.tagList}>
              {displaySkill.tags.map((tag) => (
                <span key={tag} className={styles.tag}>
                  {tag}
                </span>
              ))}
            </div>
          ) : (
            <p className={styles.emptyText}>No tags</p>
          )}
        </div>

        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Agent Compatibility</h3>
          <div className={styles.agentList}>
            {SKILLKIT_AGENTS.map((agent) => {
              const isAgentInstalled =
                isInstalled && installedSkill.agents.includes(agent.value);
              return (
                <div key={agent.value} className={styles.agentRow}>
                  <span className={styles.agentIcon}>{agent.icon}</span>
                  <span className={styles.agentName}>{agent.label}</span>
                  {isAgentInstalled ? (
                    <Check size={16} className={styles.installedIcon} />
                  ) : (
                    <X size={16} className={styles.notInstalledIcon} />
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {isInstalled && (
          <div className={styles.card}>
            <h3 className={styles.cardTitle}>Installation Info</h3>
            <div className={styles.detailRow}>
              <Clock size={16} />
              <span className={styles.detailLabel}>Installed</span>
              <span className={styles.detailValue}>
                {new Date(installedSkill.installed_at).toLocaleString()}
              </span>
            </div>
            <div className={styles.detailRow}>
              <span className={styles.detailLabel}>Path</span>
              <code className={styles.pathValue}>{installedSkill.install_path}</code>
            </div>
            <div className={styles.detailRow}>
              <span className={styles.detailLabel}>Installed for</span>
              <span className={styles.detailValue}>
                {installedSkill.agents.length} agent
                {installedSkill.agents.length !== 1 ? "s" : ""}
              </span>
            </div>
          </div>
        )}
      </div>

      {showInstallModal && displaySkill && (
        <SkillInstallModal
          skill={displaySkill}
          onClose={() => setShowInstallModal(false)}
        />
      )}

      {showTranslateModal && displaySkill && (
        <TranslateModal
          skill={installedSkill ?? displaySkill}
          onClose={() => setShowTranslateModal(false)}
        />
      )}
    </div>
  );
}
