import { useState } from "react";
import styles from "./SettingsModal.module.css";

type FontSize = "S" | "M" | "L" | "XL";
type TerminalWidth = "80" | "100" | "120" | "160";

export default function GeneralTab() {
  const [font, setFont] = useState("JetBrains Mono");
  const [fontSize, setFontSize] = useState<FontSize>("M");
  const [terminalWidth, setTerminalWidth] = useState<TerminalWidth>("120");
  const [logLevel, setLogLevel] = useState("info");
  const [maxOutputLines, setMaxOutputLines] = useState("5000");

  return (
    <div>
      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Interface Font</div>
          <div className={styles.settingDesc}>Font used in the terminal and UI</div>
        </div>
        <select className={styles.selectInput} value={font} onChange={(e) => setFont(e.target.value)}>
          <option value="JetBrains Mono">JetBrains Mono</option>
          <option value="Fira Code">Fira Code</option>
          <option value="SF Mono">SF Mono</option>
          <option value="Menlo">Menlo</option>
          <option value="Cascadia Code">Cascadia Code</option>
        </select>
      </div>

      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Font Size</div>
          <div className={styles.settingDesc}>Terminal and editor font size</div>
        </div>
        <div className={styles.radioGroup}>
          {(["S", "M", "L", "XL"] as FontSize[]).map((size) => (
            <button
              key={size}
              className={`${styles.radioBtn} ${fontSize === size ? styles.radioBtnActive : ""}`}
              onClick={() => setFontSize(size)}
            >
              {size}
            </button>
          ))}
        </div>
      </div>

      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Terminal Width</div>
          <div className={styles.settingDesc}>Default column width for terminal sessions</div>
        </div>
        <div className={styles.radioGroup}>
          {(["80", "100", "120", "160"] as TerminalWidth[]).map((width) => (
            <button
              key={width}
              className={`${styles.radioBtn} ${terminalWidth === width ? styles.radioBtnActive : ""}`}
              onClick={() => setTerminalWidth(width)}
            >
              {width}
            </button>
          ))}
        </div>
      </div>

      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Log Level</div>
          <div className={styles.settingDesc}>Minimum log level to display</div>
        </div>
        <select className={styles.selectInput} value={logLevel} onChange={(e) => setLogLevel(e.target.value)}>
          <option value="debug">Debug</option>
          <option value="info">Info</option>
          <option value="warn">Warning</option>
          <option value="error">Error</option>
        </select>
      </div>

      <div className={styles.settingRow}>
        <div>
          <div className={styles.settingLabel}>Max Output Lines</div>
          <div className={styles.settingDesc}>Maximum lines kept in terminal buffer</div>
        </div>
        <select className={styles.selectInput} value={maxOutputLines} onChange={(e) => setMaxOutputLines(e.target.value)}>
          <option value="1000">1,000</option>
          <option value="5000">5,000</option>
          <option value="10000">10,000</option>
          <option value="50000">50,000</option>
        </select>
      </div>
    </div>
  );
}
