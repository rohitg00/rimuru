import { createContext, useContext, useCallback, useState, useRef } from "react";
import { Toast, ToastType } from "./Toast";
import styles from "./Toast.module.css";

interface ToastItem {
  id: string;
  type: ToastType;
  title: string;
  description?: string;
  state: string;
}

interface ToastContextValue {
  toast: (opts: { type: ToastType; title: string; description?: string; duration?: number }) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return ctx;
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const idCounter = useRef(0);

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.map((t) => (t.id === id ? { ...t, state: "exiting" } : t)));
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 200);
  }, []);

  const toast = useCallback(
    (opts: { type: ToastType; title: string; description?: string; duration?: number }) => {
      const id = String(++idCounter.current);
      const duration = opts.duration ?? 4000;
      setToasts((prev) => {
        const next = [...prev, { id, type: opts.type, title: opts.title, description: opts.description, state: "entered" }];
        return next.length > 5 ? next.slice(-5) : next;
      });
      setTimeout(() => dismiss(id), duration);
    },
    [dismiss]
  );

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <div className={styles.container}>
        {toasts.map((t) => (
          <Toast key={t.id} id={t.id} type={t.type} title={t.title} description={t.description} onDismiss={dismiss} state={t.state} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}
