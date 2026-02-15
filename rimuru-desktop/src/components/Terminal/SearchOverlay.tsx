import { useState, useRef, useEffect, type KeyboardEvent } from "react";
import type { TerminalSearchHandle } from "./TerminalPane";
import styles from "./SearchOverlay.module.css";

interface SearchOverlayProps {
  searchRef: React.RefObject<TerminalSearchHandle | null>;
  onClose: () => void;
}

export default function SearchOverlay({ searchRef, onClose }: SearchOverlayProps) {
  const [query, setQuery] = useState("");
  const [regex, setRegex] = useState(false);
  const [caseSensitive, setCaseSensitive] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    if (query) {
      searchRef.current?.findNext(query, { regex, caseSensitive });
    }
  }, [query, regex, caseSensitive, searchRef]);

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      searchRef.current?.clearSearch();
      onClose();
    } else if (e.key === "Enter" && e.shiftKey) {
      e.preventDefault();
      searchRef.current?.findPrevious(query, { regex, caseSensitive });
    } else if (e.key === "Enter") {
      e.preventDefault();
      searchRef.current?.findNext(query, { regex, caseSensitive });
    }
  };

  return (
    <div className={styles.container}>
      <input
        ref={inputRef}
        className={styles.input}
        type="text"
        placeholder="Search terminal output..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={handleKeyDown}
      />
      <button
        className={`${styles.btn} ${regex ? styles.btnActive : ""}`}
        onClick={() => setRegex(!regex)}
        title="Use regular expression"
      >
        .*
      </button>
      <button
        className={`${styles.btn} ${caseSensitive ? styles.btnActive : ""}`}
        onClick={() => setCaseSensitive(!caseSensitive)}
        title="Match case"
      >
        Aa
      </button>
      <button
        className={styles.btn}
        onClick={() => searchRef.current?.findPrevious(query, { regex, caseSensitive })}
        title="Previous match (Shift+Enter)"
      >
        &uarr;
      </button>
      <button
        className={styles.btn}
        onClick={() => searchRef.current?.findNext(query, { regex, caseSensitive })}
        title="Next match (Enter)"
      >
        &darr;
      </button>
      <button
        className={styles.closeBtn}
        onClick={() => {
          searchRef.current?.clearSearch();
          onClose();
        }}
        title="Close (Escape)"
      >
        &times;
      </button>
    </div>
  );
}
