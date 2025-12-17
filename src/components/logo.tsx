import { ShieldCheck } from "lucide-react";
import { cn } from "../lib/utils";

type LogoProps = {
  className?: string;
  variant?: "sidebar" | "topbar";
};

export function Logo({ className, variant = "sidebar" }: LogoProps) {
  const isSidebar = variant === "sidebar";
  return (
    <div
      className={cn(
        "flex items-center gap-2 font-bold tracking-tight",
        isSidebar ? "text-foreground" : "text-foreground",
        className,
      )}
    >
      <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-soft">
        <img
          src="/sslboard_logo_250x285.png"
          alt="SSLBoard logo"
          className="h-8 w-8 object-contain"
          onError={(event) => {
            const target = event.currentTarget;
            target.style.display = "none";
          }}
        />
        <ShieldCheck className="absolute h-5 w-5 text-primary-foreground opacity-0" />
      </div>
      <div className="leading-tight">
        <div className="text-lg">SSLBoard</div>
        <div className="text-xs font-normal text-muted-foreground">
          Certificate Ops
        </div>
      </div>
    </div>
  );
}
