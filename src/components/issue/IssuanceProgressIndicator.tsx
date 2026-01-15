import { CheckCircle2, Loader2, Circle } from "lucide-react";
import { cn } from "../../lib/utils";

interface Step {
  label: string;
  status: "complete" | "active" | "pending" | "error";
}

interface IssuanceProgressIndicatorProps {
  steps: Step[];
  className?: string;
}

export function IssuanceProgressIndicator({
  steps,
  className,
}: IssuanceProgressIndicatorProps) {
  return (
    <div className={cn("w-full", className)}>
      <div className="flex items-center justify-between">
        {steps.map((step, index) => {
          const isLast = index === steps.length - 1;
          const isComplete = step.status === "complete";
          const isActive = step.status === "active";
          const isError = step.status === "error";

          return (
            <div key={index} className="flex flex-1 items-center">
              <div className="flex flex-col items-center">
                <div
                  className={cn(
                    "flex h-10 w-10 items-center justify-center rounded-full border-2 transition-colors",
                    isComplete && "border-emerald-600 bg-emerald-600",
                    isActive && "border-primary bg-primary/10",
                    isError && "border-rose-600 bg-rose-600",
                    !isComplete && !isActive && !isError && "border-muted-foreground/30 bg-background",
                  )}
                >
                  {isComplete ? (
                    <CheckCircle2 className="h-5 w-5 text-white" />
                  ) : isActive ? (
                    <Loader2 className="h-5 w-5 animate-spin text-primary" />
                  ) : isError ? (
                    <Circle className="h-5 w-5 text-white" />
                  ) : (
                    <Circle className="h-5 w-5 text-muted-foreground/30" />
                  )}
                </div>
                <div
                  className={cn(
                    "mt-2 text-xs font-medium transition-colors",
                    isComplete && "text-emerald-600",
                    isActive && "text-primary",
                    isError && "text-rose-600",
                    !isComplete && !isActive && !isError && "text-muted-foreground",
                  )}
                >
                  {step.label}
                </div>
              </div>
              {!isLast && (
                <div
                  className={cn(
                    "mx-2 h-0.5 flex-1 transition-colors",
                    isComplete ? "bg-emerald-600" : "bg-muted-foreground/30",
                  )}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
