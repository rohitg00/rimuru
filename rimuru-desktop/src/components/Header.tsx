import { RefreshCw, Moon, Sun, Search } from "lucide-react";
import { useTheme } from "@/hooks/useTheme";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { commands } from "@/lib/tauri";
import styles from "./Header.module.css";

export default function Header() {
  const { theme, setTheme } = useTheme();
  const queryClient = useQueryClient();

  const syncMutation = useMutation({
    mutationFn: commands.triggerSync,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["costs"] });
    },
  });

  const toggleTheme = () => {
    setTheme(theme === "light" ? "tokyo-night" : "light");
  };

  return (
    <header className={styles.header}>
      <div className={styles.search}>
        <Search size={18} className={styles.searchIcon} />
        <input
          type="text"
          placeholder="Search agents, sessions..."
          className={styles.searchInput}
        />
      </div>

      <div className={styles.actions}>
        <button
          className={styles.iconButton}
          onClick={() => syncMutation.mutate()}
          disabled={syncMutation.isPending}
          title="Sync model pricing"
        >
          <RefreshCw
            size={18}
            className={syncMutation.isPending ? styles.spinning : ""}
          />
        </button>

        <button
          className={styles.iconButton}
          onClick={toggleTheme}
          title="Toggle theme"
        >
          {theme === "light" ? <Moon size={18} /> : <Sun size={18} />}
        </button>
      </div>
    </header>
  );
}
