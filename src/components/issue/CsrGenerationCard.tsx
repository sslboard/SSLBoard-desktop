import { useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Label } from "../ui/label";
import { Textarea } from "../ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import {
  generateCsr,
  keyOptionToParams,
  type GenerateCsrResponse,
  type IssuanceKeyOption,
} from "../../lib/issuance";
import { normalizeError } from "../../lib/errors";

interface CsrGenerationCardProps {
  onGenerated: (result: GenerateCsrResponse) => void;
}

function formatKeyOption(option: IssuanceKeyOption): string {
  return option.replace("-", " ").toUpperCase();
}

export function CsrGenerationCard({ onGenerated }: CsrGenerationCardProps) {
  const [subject, setSubject] = useState("example.com");
  const [sansInput, setSansInput] = useState("www.example.com");
  const [keyOption, setKeyOption] = useState<IssuanceKeyOption>("rsa-2048");
  const [outputPath, setOutputPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<GenerateCsrResponse | null>(null);

  const parsedSans = useMemo(() => {
    return sansInput
      .split(/[\s,]+/)
      .map((value) => value.trim())
      .filter(Boolean);
  }, [sansInput]);

  async function handlePickPath() {
    const selection = await save({
      title: "Save CSR file",
      filters: [{ name: "CSR", extensions: ["csr", "pem"] }],
    });
    if (typeof selection === "string") {
      setOutputPath(selection);
    }
  }

  async function handleGenerate() {
    setError(null);
    setResult(null);
    if (!subject.trim()) {
      setError("Enter a subject (common name) before generating the CSR.");
      return;
    }
    if (!outputPath) {
      setError("Choose a destination path for the CSR file.");
      return;
    }

    setLoading(true);
    try {
      const keyParams = keyOptionToParams(keyOption);
      const response = await generateCsr({
        subject: subject.trim(),
        sans: parsedSans,
        output_path: outputPath,
        ...keyParams,
      });
      setResult(response);
      onGenerated(response);
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <Card className="shadow-soft">
      <CardHeader className="flex-row items-start justify-between gap-4 space-y-0">
        <div>
          <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
            CSR creation
          </div>
          <CardTitle className="text-xl font-bold">Generate a CSR</CardTitle>
          <p className="mt-2 text-sm text-muted-foreground">
            Create a CSR backed by a managed private key stored in the core. Use the CSR later for
            issuance with your issuer of choice.
          </p>
        </div>
        <div className="hidden rounded-lg border bg-muted px-3 py-2 text-xs text-muted-foreground sm:block">
          Keys stay encrypted in the vault.
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground" htmlFor="csr-subject">
            Subject (common name)
          </Label>
          <Textarea
            id="csr-subject"
            value={subject}
            onChange={(e) => setSubject(e.target.value.normalize("NFC"))}
            rows={1}
            placeholder="example.com"
          />
        </div>

        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground" htmlFor="csr-sans">
            Subject Alternative Names (optional)
          </Label>
          <Textarea
            id="csr-sans"
            value={sansInput}
            onChange={(e) => setSansInput(e.target.value.normalize("NFC"))}
            rows={2}
            placeholder="www.example.com, api.example.com"
          />
          <p className="text-xs text-muted-foreground">
            Comma or newline separated. Leave blank for CN-only CSRs.
          </p>
        </div>

        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground">Key algorithm</Label>
          <Select
            value={keyOption}
            onValueChange={(value) => setKeyOption(value as IssuanceKeyOption)}
            disabled={loading}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select key algorithm" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="rsa-2048">RSA 2048</SelectItem>
              <SelectItem value="rsa-3072">RSA 3072</SelectItem>
              <SelectItem value="rsa-4096">RSA 4096</SelectItem>
              <SelectItem value="ecdsa-p256">ECDSA P-256</SelectItem>
              <SelectItem value="ecdsa-p384">ECDSA P-384</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            Selected key: {formatKeyOption(keyOption)}.
          </p>
        </div>

        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground">Output path</Label>
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="outline" onClick={handlePickPath} disabled={loading}>
              Choose CSR destination
            </Button>
            {outputPath ? (
              <span className="text-xs text-muted-foreground">{outputPath}</span>
            ) : (
              <span className="text-xs text-muted-foreground">No output path selected.</span>
            )}
          </div>
        </div>

        {error ? (
          <div className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700">
            {error}
          </div>
        ) : null}

        {result ? (
          <div className="rounded-lg border bg-muted/40 px-3 py-2 text-xs text-muted-foreground">
            CSR generated at {result.csr_path}. Key stored as {result.managed_key_ref}.
          </div>
        ) : null}

        {result?.result.warnings.length ? (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-900">
            {result.result.warnings.join(" ")}
          </div>
        ) : null}

        <div className="flex flex-wrap gap-3">
          <Button onClick={() => void handleGenerate()} disabled={loading}>
            {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Generate CSR
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
