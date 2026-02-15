import { useState, useEffect, useCallback } from "react";
import { X } from "lucide-react";
import { commands } from "@/lib/tauri";
import { useAnimatedUnmount } from "@/hooks/useAnimatedUnmount";
import styles from "./RemoteControlModal.module.css";

interface RemoteControlModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface RemoteStatus {
  running: boolean;
  url: string | null;
  qr_svg: string | null;
}

export default function RemoteControlModal({ isOpen, onClose }: RemoteControlModalProps) {
  const { shouldRender, animationState } = useAnimatedUnmount(isOpen, 200);
  const [status, setStatus] = useState<RemoteStatus>({ running: false, url: null, qr_svg: null });
  const [port, setPort] = useState("3847");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const fetchStatus = useCallback(async () => {
    try {
      const s = await commands.getRemoteStatus();
      setStatus(s);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      fetchStatus();
    }
  }, [isOpen, fetchStatus]);

  const handleStart = async () => {
    setLoading(true);
    setError(null);
    try {
      const s = await commands.startRemoteServer(parseInt(port, 10) || 3847);
      setStatus(s);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleStop = async () => {
    setLoading(true);
    setError(null);
    try {
      await commands.stopRemoteServer();
      setStatus({ running: false, url: null, qr_svg: null });
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = () => {
    if (status.url) {
      navigator.clipboard.writeText(status.url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (!shouldRender) return null;

  return (
    <div className={styles.overlay} data-state={animationState} onClick={onClose}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <h2 className={styles.title}>Remote Control</h2>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close dialog">
            <X size={18} />
          </button>
        </div>

        <div className={styles.body}>
          <div className={styles.statusRow}>
            <span className={`${styles.statusDot} ${status.running ? styles.statusRunning : styles.statusStopped}`} />
            {status.running ? "Server running" : "Server stopped"}
          </div>

          {status.running && status.qr_svg && (
            <div
              className={styles.qrContainer}
              dangerouslySetInnerHTML={{ __html: status.qr_svg }}
            />
          )}

          {status.running && status.url && (
            <>
              <div className={styles.url}>{status.url}</div>
              <button className={styles.copyBtn} onClick={handleCopy}>
                {copied ? "Copied!" : "Copy URL"}
              </button>
            </>
          )}

          {error && <div className={styles.error}>{error}</div>}
        </div>

        <div className={styles.controls}>
          {!status.running ? (
            <>
              <div className={styles.portRow}>
                <span className={styles.portLabel}>Port</span>
                <input
                  className={styles.portInput}
                  type="number"
                  value={port}
                  onChange={(e) => setPort(e.target.value)}
                  min={1024}
                  max={65535}
                />
              </div>
              <button className={styles.startBtn} onClick={handleStart} disabled={loading}>
                {loading ? "Starting..." : "Start Server"}
              </button>
            </>
          ) : (
            <button className={styles.stopBtn} onClick={handleStop} disabled={loading}>
              {loading ? "Stopping..." : "Stop Server"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
