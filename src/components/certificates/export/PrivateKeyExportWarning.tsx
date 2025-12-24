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
        <input
          id="include-key"
          type="checkbox"
          className="mt-1 h-4 w-4"
          disabled={!keyAvailable}
          checked={includeKey}
          onChange={(event) => {
            onIncludeKeyChange(event.target.checked);
            if (!event.target.checked) {
              onConfirmKeyExportChange(false);
            }
          }}
        />
        <label htmlFor="include-key" className="text-sm font-semibold">
          Include private key (`privkey.pem`)
        </label>
      </div>
      {!keyAvailable && (
        <div className="mt-2 text-xs text-amber-700">
          Private key export is unavailable for certificates without a
          managed key reference.
        </div>
      )}
      {includeKey && (
        <div className="mt-3 flex items-start gap-2 rounded-md bg-amber-100/80 p-2">
          <input
            id="confirm-key"
            type="checkbox"
            className="mt-1 h-4 w-4"
            checked={confirmKeyExport}
            onChange={(event) => onConfirmKeyExportChange(event.target.checked)}
          />
          <label htmlFor="confirm-key" className="text-xs text-amber-900">
            I understand exporting private keys is sensitive and I accept
            the risk.
          </label>
        </div>
      )}
    </div>
  );
}

