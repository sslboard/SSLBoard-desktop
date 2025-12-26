import { Checkbox } from "../../ui/checkbox";
import { Label } from "../../ui/label";

interface PrivateKeyExportWarningProps {
  keyAvailable: boolean;
  includeKey: boolean;
  confirmKeyExport: boolean;
  onIncludeKeyChange: (include: boolean) => void;
  onConfirmKeyExportChange: (confirm: boolean) => void;
}

export function PrivateKeyExportWarning({
  keyAvailable,
  includeKey,
  confirmKeyExport,
  onIncludeKeyChange,
  onConfirmKeyExportChange,
}: PrivateKeyExportWarningProps) {
  return (
    <div className="rounded-lg border border-amber-200/60 bg-amber-50/70 p-4 text-sm text-amber-900">
      <div className="font-semibold">Private key export</div>
      <div className="mt-2 text-xs text-amber-800/80">
        Exporting private keys can expose sensitive material. Only proceed
        if you trust the destination.
      </div>
      <div className="mt-3 flex items-start gap-2">
        <Checkbox
          id="include-key"
          className="mt-1 border-amber-400"
          disabled={!keyAvailable}
          checked={includeKey}
          onCheckedChange={(checked) => {
            const shouldInclude = checked === true;
            onIncludeKeyChange(shouldInclude);
            if (!shouldInclude) {
              onConfirmKeyExportChange(false);
            }
          }}
        />
        <Label htmlFor="include-key" className="text-sm font-semibold text-amber-900">
          Include private key (`privkey.pem`)
        </Label>
      </div>
      {!keyAvailable && (
        <div className="mt-2 text-xs text-amber-700">
          Private key export is unavailable for certificates without a
          managed key reference.
        </div>
      )}
      {includeKey && (
        <div className="mt-3 flex items-start gap-2 rounded-md bg-amber-100/80 p-2">
          <Checkbox
            id="confirm-key"
            className="mt-1 border-amber-400"
            checked={confirmKeyExport}
            onCheckedChange={(checked) =>
              onConfirmKeyExportChange(checked === true)
            }
          />
          <Label htmlFor="confirm-key" className="text-xs text-amber-900">
            I understand exporting private keys is sensitive and I accept
            the risk.
          </Label>
        </div>
      )}
    </div>
  );
}
