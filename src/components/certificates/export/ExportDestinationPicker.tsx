import { Button } from "../../ui/button";

interface ExportDestinationPickerProps {
  destinationDir: string | null;
  folderName: string;
  onSelectDestination: () => void;
  onFolderNameChange: (name: string) => void;
}

export function ExportDestinationPicker({
  destinationDir,
  folderName,
  onSelectDestination,
  onFolderNameChange,
}: ExportDestinationPickerProps) {
  return (
    <div className="rounded-lg border bg-muted/40 p-4">
      <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        Destination
      </div>
      <div className="mt-3 flex flex-col gap-3 sm:flex-row sm:items-center">
        <Button variant="outline" onClick={onSelectDestination}>
          Choose folder
        </Button>
        <div className="text-sm text-muted-foreground">
          {destinationDir ?? "No folder selected"}
        </div>
      </div>
      <div className="mt-3">
        <label className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Export subfolder name
        </label>
        <input
          value={folderName}
          onChange={(event) => onFolderNameChange(event.target.value)}
          className="mt-2 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground"
        />
      </div>
    </div>
  );
}

