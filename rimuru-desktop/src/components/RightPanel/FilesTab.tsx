import React, { useState, useEffect, useCallback } from "react";
import { Folder, FolderOpen, FileText, Eye, EyeOff, RefreshCw } from "lucide-react";
import { commands, DirEntry, DirectoryStats } from "@/lib/tauri";
import { Tooltip } from "@/components/Tooltip/Tooltip";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import styles from "./FilesTab.module.css";

interface FilesTabProps {
  workingDir?: string;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export const FilesTab: React.FC<FilesTabProps> = ({ workingDir }) => {
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());
  const [entries, setEntries] = useState<Map<string, DirEntry[]>>(new Map());
  const [stats, setStats] = useState<DirectoryStats | null>(null);
  const [showHidden, setShowHidden] = useState(false);
  const [loading, setLoading] = useState(false);

  const loadDir = useCallback(async (path: string) => {
    try {
      const result = await commands.readDirectory(path);
      setEntries((prev) => new Map(prev).set(path, result));
    } catch {
      /* ignore */
    }
  }, []);

  const refresh = useCallback(async () => {
    if (!workingDir) return;
    setLoading(true);
    try {
      const [rootEntries, dirStats] = await Promise.all([
        commands.readDirectory(workingDir),
        commands.getDirectoryStats(workingDir),
      ]);
      setEntries(new Map([[workingDir, rootEntries]]));
      setExpandedDirs(new Set());
      setStats(dirStats);
    } catch {
      /* ignore */
    } finally {
      setLoading(false);
    }
  }, [workingDir]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const toggleDir = (path: string) => {
    setExpandedDirs((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
        if (!entries.has(path)) loadDir(path);
      }
      return next;
    });
  };

  const filterEntries = (items: DirEntry[]) =>
    showHidden ? items : items.filter((e) => !e.name.startsWith("."));

  const renderTree = (parentPath: string, depth: number) => {
    const items = entries.get(parentPath);
    if (!items) return null;

    const filtered = filterEntries(items);
    const sorted = [...filtered].sort((a, b) => {
      if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });

    return sorted.map((entry) => {
      const fullPath = `${parentPath}/${entry.name}`;
      const isExpanded = expandedDirs.has(fullPath);

      return (
        <React.Fragment key={fullPath}>
          <div
            className={`${styles.entry} ${entry.is_dir ? styles.entryDir : ""}`}
            onClick={() => entry.is_dir && toggleDir(fullPath)}
          >
            <span className={styles.indent} style={{ width: depth * 16 }} />
            {entry.is_dir ? (
              isExpanded ? <FolderOpen size={14} /> : <Folder size={14} />
            ) : (
              <FileText size={14} />
            )}
            <span>{entry.name}</span>
          </div>
          {entry.is_dir && isExpanded && renderTree(fullPath, depth + 1)}
        </React.Fragment>
      );
    });
  };

  if (!workingDir) {
    return <EmptyState icon={FolderOpen} title="No working directory" description="Open a project to browse files" />;
  }

  return (
    <div className={styles.container}>
      <div className={styles.toolbar}>
        <div className={styles.stats}>
          {stats && (
            <>
              <span>{stats.file_count} files</span>
              <span>{stats.folder_count} folders</span>
              <span>{formatSize(stats.total_size)}</span>
            </>
          )}
        </div>
        <div className={styles.actions}>
          <Tooltip content="Show hidden files">
            <button
              className={`${styles.iconBtn} ${showHidden ? styles.iconBtnActive : ""}`}
              onClick={() => setShowHidden(!showHidden)}
              aria-label="Toggle hidden files"
            >
              {showHidden ? <Eye size={14} /> : <EyeOff size={14} />}
            </button>
          </Tooltip>
          <Tooltip content="Refresh" shortcut="\u2318R">
            <button className={styles.iconBtn} onClick={refresh} disabled={loading} aria-label="Refresh file tree">
              <RefreshCw size={14} />
            </button>
          </Tooltip>
        </div>
      </div>
      <div className={styles.tree}>{renderTree(workingDir, 0)}</div>
    </div>
  );
};
