import { Component, type ReactNode } from "react";

interface Props { children: ReactNode; fallback?: ReactNode }
interface State { hasError: boolean; error: string }

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: "" };
  }
  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error: error.message };
  }
  render() {
    if (this.state.hasError) {
      return this.props.fallback ?? (
        <div className="rounded-xl border border-[var(--error)]/30 bg-[var(--error)]/5 p-8 text-center">
          <p className="text-[var(--error)] font-medium mb-2">Something went wrong</p>
          <p className="text-xs text-[var(--text-secondary)]">{this.state.error}</p>
          <button onClick={() => this.setState({ hasError: false, error: "" })} className="mt-4 px-4 py-2 text-xs rounded-lg bg-[var(--bg-secondary)] text-[var(--text-primary)] border border-[var(--border)]">
            Try Again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
