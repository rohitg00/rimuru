import { useEffect, useState } from "react";
import { Routes, Route, useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import Layout from "@/components/Layout";
import Dashboard from "@/views/Dashboard";
import Agents from "@/views/Agents";
import AgentDetails from "@/views/AgentDetails";
import Sessions from "@/views/Sessions";
import SessionDetails from "@/views/SessionDetails";
import Costs from "@/views/Costs";
import Metrics from "@/views/Metrics";
import Settings from "@/views/Settings";
import Skills from "@/views/Skills";
import SkillDetails from "@/views/SkillDetails";
import Plugins from "@/views/Plugins";
import Hooks from "@/views/Hooks";
import Orchestrate from "@/views/Orchestrate";
import GroupChat from "@/views/GroupChat";
import Playbooks from "@/views/Playbooks";
import UsageDashboard from "@/views/UsageDashboard";
import { Onboarding, useOnboarding } from "@/components/Onboarding";
import { AboutDialog, useAboutDialog } from "@/components/AboutDialog";
import { KeyboardShortcutsHelp } from "@/components/KeyboardShortcutsHelp";
import { useGlobalShortcuts } from "@/hooks/useKeyboardShortcuts";
import { MenuBar } from "@/components/MenuBar";
import SettingsModal from "@/components/Settings/SettingsModal";

function App() {
  const navigate = useNavigate();
  const { showOnboarding, completeOnboarding, resetOnboarding } = useOnboarding();
  const aboutDialog = useAboutDialog();
  const shortcuts = useGlobalShortcuts();
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    const unsubscribeNavigate = listen<string>("navigate", (event) => {
      navigate(event.payload);
    });

    const unsubscribeAbout = listen("show-about", () => {
      aboutDialog.open();
    });

    const settingsHandler = () => setShowSettings(true);
    window.addEventListener("show-settings-modal", settingsHandler);

    return () => {
      unsubscribeNavigate.then((fn) => fn());
      unsubscribeAbout.then((fn) => fn());
      window.removeEventListener("show-settings-modal", settingsHandler);
    };
  }, [navigate, aboutDialog]);

  const handleShowShortcuts = () => {
    window.dispatchEvent(new CustomEvent("show-shortcuts-help"));
  };

  return (
    <>
      <MenuBar
        onShowAbout={aboutDialog.open}
        onShowSettings={() => setShowSettings(true)}
        onShowShortcuts={handleShowShortcuts}
        onResetOnboarding={resetOnboarding}
      />
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/agents" element={<Agents />} />
          <Route path="/agents/:id" element={<AgentDetails />} />
          <Route path="/orchestrate" element={<Orchestrate />} />
          <Route path="/groupchat" element={<GroupChat />} />
          <Route path="/playbooks" element={<Playbooks />} />
          <Route path="/sessions" element={<Sessions />} />
          <Route path="/sessions/:id" element={<SessionDetails />} />
          <Route path="/costs" element={<Costs />} />
          <Route path="/metrics" element={<Metrics />} />
          <Route path="/skills" element={<Skills />} />
          <Route path="/skills/:id" element={<SkillDetails />} />
          <Route path="/plugins" element={<Plugins />} />
          <Route path="/hooks" element={<Hooks />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/usage" element={<UsageDashboard />} />
        </Routes>
      </Layout>

      {showOnboarding && <Onboarding onComplete={completeOnboarding} />}
      <AboutDialog isOpen={aboutDialog.isOpen} onClose={aboutDialog.close} />
      <KeyboardShortcutsHelp shortcuts={shortcuts} />
      <SettingsModal isOpen={showSettings} onClose={() => setShowSettings(false)} />
    </>
  );
}

export default App;
