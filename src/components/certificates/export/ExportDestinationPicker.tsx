import { Button } from "../../ui/button";
import { Input } from "../../ui/input";
import { Label } from "../../ui/label";

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
        <Label
          htmlFor="export-subfolder-name"
          className="text-xs font-semibold uppercase tracking-wide text-muted-foreground"
        >
          Export subfolder name
        </Label>
        <Input
          id="export-subfolder-name"
          value={folderName}
          onChange={(event) => onFolderNameChange(event.target.value)}
          className="mt-2"
        />
      </div>
    </div>
  );
}
