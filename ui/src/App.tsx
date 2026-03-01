import "./index.css";
import { useState, useEffect, useCallback } from "react";
import Layout from "./components/Layout";
import ErrorBoundary from "./components/ErrorBoundary";
import Dashboard from "./pages/Dashboard";
import Agents from "./pages/Agents";
import Sessions from "./pages/Sessions";
import Costs from "./pages/Costs";
import Models from "./pages/Models";
import Metrics from "./pages/Metrics";
import Plugins from "./pages/Plugins";
import Hooks from "./pages/Hooks";
import Settings from "./pages/Settings";
import Terminal from "./pages/Terminal";
import McpServers from "./pages/McpServers";
import { applyTheme, getStoredTheme } from "./components/ThemeSwitcher";

const PAGES: Record<string, React.ComponentType> = {
  "": Dashboard,
  agents: Agents,
  sessions: Sessions,
  costs: Costs,
  models: Models,
  metrics: Metrics,
  plugins: Plugins,
  hooks: Hooks,
  mcp: McpServers,
  settings: Settings,
  terminal: Terminal,
};

function getHashPage(): string {
  const hash = window.location.hash.replace("#/", "").replace("#", "");
  return hash;
}

export default function App() {
  const [page, setPage] = useState(getHashPage);

  useEffect(() => {
    applyTheme(getStoredTheme());
  }, []);

  useEffect(() => {
    function onHashChange() {
      setPage(getHashPage());
    }
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const navigate = useCallback((path: string) => {
    window.location.hash = `#/${path}`;
  }, []);

  const PageComponent = PAGES[page] ?? Dashboard;

  return (
    <Layout currentPage={page} navigate={navigate}>
      <ErrorBoundary key={page}>
        <PageComponent />
      </ErrorBoundary>
    </Layout>
  );
}
