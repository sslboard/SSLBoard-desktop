import { NavLink } from "react-router-dom";
import { cn } from "../../lib/utils";
import { Logo } from "../logo";
import { buttonVariants } from "../ui/button";
import type { NavItem } from "./app-shell";

type TopbarProps = {
  navItems: NavItem[];
  vaultUnlocked: boolean;
};

export function Topbar({
  navItems,
  vaultUnlocked,
}: TopbarProps) {
  const vaultLabel = vaultUnlocked ? "Vault unlocked" : "Vault locked";

  return (
    <header className="sticky top-0 z-30 w-full border-b bg-card/95 px-4 backdrop-blur">
      <div className="flex h-16 items-center justify-between gap-3">
        <div className="flex min-w-0 flex-1 items-center gap-4">
          <Logo variant="topbar" className="shrink-0" />
          <nav className="flex max-w-full flex-1 items-center gap-1 overflow-x-auto text-base">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === "/settings"}
                className={({ isActive }) =>
                  cn(
                    buttonVariants({ variant: "ghost", size: "sm" }),
                    "h-9 gap-2 px-4",
                    isActive
                      ? "bg-muted text-foreground"
                      : "text-muted-foreground hover:bg-muted/70",
                  )
                }
              >
                {item.icon}
                <span className="whitespace-nowrap font-medium">
                  {item.label}
                </span>
              </NavLink>
            ))}
          </nav>
        </div>
        <div className="flex items-center gap-3 text-base text-muted-foreground">
          <div className="hidden items-center gap-2 sm:flex">
            <span
              className={cn(
                "h-2.5 w-2.5 rounded-full",
                vaultUnlocked ? "bg-emerald-500" : "bg-amber-500",
              )}
            />
            <span>{vaultLabel}</span>
          </div>
        </div>
      </div>
    </header>
  );
}
