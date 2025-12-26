import type { ExportBundle } from "../../../lib/certificates";
import { Button } from "../../ui/button";

const bundleOptions: { value: ExportBundle; label: string; hint: string }[] = [
  { value: "cert", label: "Certificate", hint: "Leaf certificate only" },
  { value: "chain", label: "Chain", hint: "Issuer chain only" },
  { value: "fullchain", label: "Full chain", hint: "Leaf + issuer chain" },
];

interface ExportBundleSelectorProps {
  bundle: ExportBundle;
  onBundleChange: (bundle: ExportBundle) => void;
}

export function ExportBundleSelector({
  bundle,
  onBundleChange,
}: ExportBundleSelectorProps) {
  return (
    <div>
      <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        Bundle selection
      </div>
      <div className="mt-2 grid gap-2 sm:grid-cols-3">
        {bundleOptions.map((option) => (
          <Button
            key={option.value}
            type="button"
            variant="outline"
            onClick={() => onBundleChange(option.value)}
            className={`h-auto w-full flex-col items-start justify-start rounded-lg px-3 py-2 text-left font-normal ${
              bundle === option.value
                ? "border-primary bg-primary/10 text-primary"
                : "border-border bg-background text-foreground hover:bg-muted/50"
            }`}
          >
            <div className="font-semibold">{option.label}</div>
            <div className="text-xs text-muted-foreground">
              {option.hint}
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}
