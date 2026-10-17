import { useState, type ReactNode } from "react";
import { Eye, LogOut, Menu, Package, Settings, Trophy } from "lucide-react";
import type { Admin } from "../api";
import { Button } from "../components/ui/button";
import { Card } from "../components/ui/card";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "../components/ui/sheet";
export type AdminPage = "products" | "topicReaders" | "settings";
const PAGE_TITLES: Record<AdminPage, { title: string; subtitle: string }> = {
  products: { title: "商品管理", subtitle: "会员微信道具映射" },
  topicReaders: { title: "话题读者", subtitle: "今日话题被哪些登录用户阅读过" },
  settings: { title: "设置", subtitle: "管理员账号与密码" },
};
export function AdminShell({
  admin,
  themeControl,
  onLogout,
  loggingOut,
  page,
  onNavigate,
  navigationLocked,
  children,
}: {
  admin: Admin;
  themeControl: ReactNode;
  onLogout: () => void;
  loggingOut: boolean;
  page: AdminPage;
  onNavigate: (page: AdminPage) => void;
  navigationLocked: boolean;
  children: ReactNode;
}) {
  const [drawer, setDrawer] = useState(false);
  const nav = (
    <nav aria-label="后台导航" className="sidebar-navigation">
      <p className="sidebar-label">业务管理</p>
      <Button
        variant="ghost"
        className="sidebar-nav-item"
        aria-current={page === "products" ? "page" : undefined}
        disabled={navigationLocked}
        onClick={() => {
          onNavigate("products");
          setDrawer(false);
        }}
      >
        <Package />
        商品管理
      </Button>
      <Button
        variant="ghost"
        className="sidebar-nav-item"
        aria-current={page === "topicReaders" ? "page" : undefined}
        disabled={navigationLocked}
        onClick={() => {
          onNavigate("topicReaders");
          setDrawer(false);
        }}
      >
        <Eye />
        话题读者
      </Button>
      <Button
        variant="ghost"
        className="sidebar-nav-item"
        aria-current={page === "settings" ? "page" : undefined}
        disabled={navigationLocked}
        onClick={() => {
          onNavigate("settings");
          setDrawer(false);
        }}
      >
        <Settings />
        设置
      </Button>
    </nav>
  );
  return (
    <div className="admin-shell">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <Trophy aria-hidden="true" />
          <div>
            <strong>足球洞察</strong>
            <span>管理后台</span>
          </div>
        </div>
        {nav}
        <p className="sidebar-footer">Football Insight</p>
      </aside>
      <div className="workspace">
        <header className="workspace-header">
          <div className="header-title">
            <Button
              variant="ghost"
              size="icon"
              className="mobile-menu"
              aria-label="打开导航"
              disabled={navigationLocked}
              onClick={() => setDrawer(true)}
            >
              <Menu />
            </Button>
            <div>
              <h1>{PAGE_TITLES[page].title}</h1>
              <p>{PAGE_TITLES[page].subtitle}</p>
            </div>
          </div>
          <div className="header-actions">
            <span className="admin-identity" title={admin.username}>
              {admin.display_name || admin.username}
              <small>{admin.role}</small>
            </span>
            {themeControl}
            <Button
              variant="secondary"
              onClick={onLogout}
              disabled={loggingOut || navigationLocked}
            >
              <LogOut />
              {loggingOut ? "退出中…" : "退出"}
            </Button>
          </div>
        </header>
        <main className="workspace-content">
          <Card className="main-content-card">{children}</Card>
        </main>
      </div>
      <Sheet open={drawer} onOpenChange={setDrawer}>
        <SheetContent side="left" className="navigation-drawer">
          <SheetHeader>
            <SheetTitle>足球洞察 · 管理后台</SheetTitle>
            <SheetDescription>业务导航</SheetDescription>
          </SheetHeader>
          {nav}
        </SheetContent>
      </Sheet>
    </div>
  );
}
