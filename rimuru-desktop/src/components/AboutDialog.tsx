import { useEffect, useState } from "react";
import { X, ExternalLink, Github, Heart } from "lucide-react";
import styles from "./AboutDialog.module.css";

const VERSION = "0.1.0";
const AUTHOR = "Rohit Ghumare";
const REPO_URL = "https://github.com/rohitg00/rimuru";
const LICENSE = "Apache 2.0";

interface AboutDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AboutDialog({ isOpen, onClose }: AboutDialogProps) {
  useEffect(() => {
    if (isOpen) {
      const handleEsc = (e: KeyboardEvent) => {
        if (e.key === "Escape") {
          onClose();
        }
      };
      window.addEventListener("keydown", handleEsc);
      return () => window.removeEventListener("keydown", handleEsc);
    }
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
        <button
          className={styles.closeBtn}
          onClick={onClose}
          aria-label="Close"
        >
          <X size={20} />
        </button>

        <div className={styles.content}>
          <div className={styles.logo}>
            <img src="/rimuru.svg" alt="Rimuru" width={80} height={80} />
          </div>

          <h1 className={styles.title}>Rimuru</h1>
          <p className={styles.subtitle}>AI Agent Orchestration Platform</p>

          <div className={styles.version}>
            <span className={styles.versionLabel}>Version</span>
            <span className={styles.versionNumber}>{VERSION}</span>
          </div>

          <p className={styles.description}>
            A unified platform to manage, monitor, and analyze costs across
            multiple AI coding agents including Claude Code, GitHub Copilot,
            Cursor, Codex, Goose, and OpenCode.
          </p>

          <div className={styles.details}>
            <div className={styles.detail}>
              <span className={styles.detailLabel}>Author</span>
              <span className={styles.detailValue}>{AUTHOR}</span>
            </div>
            <div className={styles.detail}>
              <span className={styles.detailLabel}>License</span>
              <span className={styles.detailValue}>{LICENSE}</span>
            </div>
          </div>

          <div className={styles.links}>
            <a
              href={REPO_URL}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.link}
            >
              <Github size={18} />
              GitHub Repository
              <ExternalLink size={14} className={styles.externalIcon} />
            </a>
            <a
              href={`${REPO_URL}/issues`}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.link}
            >
              Report an Issue
              <ExternalLink size={14} className={styles.externalIcon} />
            </a>
            <a
              href={`${REPO_URL}#contributing`}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.link}
            >
              <Heart size={18} />
              Contribute
              <ExternalLink size={14} className={styles.externalIcon} />
            </a>
          </div>

          <div className={styles.credits}>
            <p>
              Built with <Heart size={14} className={styles.heartIcon} /> using
              Tauri, React, and Rust
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

export function useAboutDialog() {
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    const handleShowAbout = () => setIsOpen(true);
    window.addEventListener("show-about-dialog", handleShowAbout);
    return () => window.removeEventListener("show-about-dialog", handleShowAbout);
  }, []);

  return {
    isOpen,
    open: () => setIsOpen(true),
    close: () => setIsOpen(false),
  };
}
