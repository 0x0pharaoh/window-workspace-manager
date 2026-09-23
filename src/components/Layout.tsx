import type { ReactNode } from "react";
import {
  Activity,
  AppWindow,
  Home,
  LayoutGrid,
  Monitor,
  Moon,
  Settings,
  Sun,
} from "lucide-react";
import type { ViewKey } from "@/types";
import { cn } from "@/components/ui";

const NAV: { key: ViewKey; label: string; icon: typeof Home }[] = [
  { key: "home", label: "Home", icon: Home },
  { key: "workspaces", label: "Workspaces", icon: LayoutGrid },
  { key: "editor", label: "Layout Editor", icon: Monitor },
  { key: "apps", label: "Applications", icon: AppWindow },
  { key: "sessions", label: "Active Sessions", icon: Activity },
  { key: "settings", label: "Settings", icon: Settings },
];

export function Layout({
  view,
  onNavigate,
  dark,
  onToggleDark,
  children,
}: {
  view: ViewKey;
  onNavigate: (v: ViewKey) => void;
  dark: boolean;
  onToggleDark: () => void;
  children: ReactNode;
}) {
  return (
    <div className="flex h-full bg-slate-50 text-slate-900 dark:bg-slate-950 dark:text-slate-100">
      <aside className="flex w-56 shrink-0 flex-col border-r border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900">
        <div className="flex items-center gap-2 px-4 py-4">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-sky-600 font-bold text-white">
            W
          </div>
          <div>
            <div className="text-sm font-bold leading-tight">Workset</div>
            <div className="text-[11px] text-slate-500 dark:text-slate-400">Workspace Manager</div>
          </div>
        </div>
        <nav aria-label="Main navigation" className="flex flex-1 flex-col gap-1 px-2">
          {NAV.map((item) => {
            const Icon = item.icon;
            const active = view === item.key;
            return (
              <button
                key={item.key}
                type="button"
                aria-current={active ? "page" : undefined}
                onClick={() => onNavigate(item.key)}
                className={cn(
                  "flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                  active
                    ? "bg-sky-100 text-sky-900 dark:bg-sky-900/40 dark:text-sky-100"
                    : "text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800",
                )}
              >
                <Icon size={17} aria-hidden />
                {item.label}
              </button>
            );
          })}
        </nav>
        <div className="border-t border-slate-200 p-3 dark:border-slate-800">
          <button
            type="button"
            onClick={onToggleDark}
            aria-label={dark ? "Switch to light theme" : "Switch to dark theme"}
            className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800"
          >
            {dark ? <Sun size={16} aria-hidden /> : <Moon size={16} aria-hidden />}
            {dark ? "Light mode" : "Dark mode"}
          </button>
        </div>
      </aside>
      <main className="min-w-0 flex-1 overflow-auto">
        <div className="mx-auto max-w-6xl p-6">{children}</div>
      </main>
    </div>
  );
}
