import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Bot,
  Terminal,
  Clock,
  DollarSign,
  Activity,
  Settings,
  Puzzle,
  Package,
  Webhook,
  BookOpen,
  MessageSquare,
} from "lucide-react";
import styles from "./Sidebar.module.css";
import clsx from "clsx";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/agents", icon: Bot, label: "Agents" },
  { to: "/orchestrate", icon: Terminal, label: "Orchestrate" },
  { to: "/playbooks", icon: BookOpen, label: "Playbooks" },
  { to: "/groupchat", icon: MessageSquare, label: "Group Chat" },
  { to: "/sessions", icon: Clock, label: "Sessions" },
  { to: "/skills", icon: Puzzle, label: "Skills" },
  { to: "/plugins", icon: Package, label: "Plugins" },
  { to: "/hooks", icon: Webhook, label: "Hooks" },
  { to: "/costs", icon: DollarSign, label: "Costs" },
  { to: "/metrics", icon: Activity, label: "Metrics" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  return (
    <aside className={styles.sidebar}>
      <div className={styles.logo}>
        <span className={styles.logoIcon}>R</span>
        <span className={styles.logoText}>Rimuru</span>
      </div>

      <nav className={styles.nav}>
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              clsx(styles.navItem, isActive && styles.active)
            }
            end={item.to === "/"}
          >
            <item.icon size={20} />
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <div className={styles.footer}>
        <span className={styles.version}>v0.1.0</span>
      </div>
    </aside>
  );
}
