import { ReactNode } from "react";
import { useLocation } from "react-router-dom";
import Sidebar from "./Sidebar";
import Header from "./Header";
import StatusBar from "./StatusBar";
import styles from "./Layout.module.css";

interface LayoutProps {
  children: ReactNode;
  noScroll?: boolean;
}

const NO_SCROLL_ROUTES = ["/orchestrate", "/groupchat"];

export default function Layout({ children, noScroll }: LayoutProps) {
  const location = useLocation();
  const useNoScroll = noScroll || NO_SCROLL_ROUTES.includes(location.pathname);

  return (
    <div className={styles.layout}>
      <Sidebar />
      <div className={styles.main}>
        <Header />
        <main className={useNoScroll ? styles.contentNoScroll : styles.content}>{children}</main>
        <StatusBar />
      </div>
    </div>
  );
}
