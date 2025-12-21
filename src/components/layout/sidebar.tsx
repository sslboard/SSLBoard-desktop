import { NavLink } from "react-router-dom";
import { cn } from "../../lib/utils";
import { Logo } from "../logo";

export type NavItem = {
  label: string;
  description?: string;
  to: string;
  icon: React.ReactNode;
};

type SidebarProps = {
  items: NavItem[];
  onNavigate?: () => void;
};

export function Sidebar({ items, onNavigate }: SidebarProps) {
  return (
    <aside className="flex h-full min-w-[240px] flex-col gap-6 border-r bg-card px-4 py-6">
      <Logo />
      <nav className="flex flex-1 flex-col gap-1 text-sm">
        {items.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            onClick={onNavigate}
            end={item.to === "/settings"}
            className={({ isActive }) =>
              cn(
                "group flex items-start gap-3 rounded-xl px-3 py-3 transition-colors",
                isActive
                  ? "bg-primary/10 text-primary"
                  : "text-foreground hover:bg-muted",
              )
            }
          >
            <span className="mt-1">{item.icon}</span>
            <span className="flex flex-col gap-0.5">
              <span className="font-semibold">{item.label}</span>
              {item.description ? (
                <span className="text-xs text-muted-foreground">
                  {item.description}
                </span>
              ) : null}
            </span>
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
