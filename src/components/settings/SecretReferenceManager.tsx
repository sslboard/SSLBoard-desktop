import { useMemo, useState, type FormEvent } from "react";
import { useForm } from "react-hook-form";
import { KeyRound, Plus, RefreshCw, Shield, Trash2 } from "lucide-react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "../ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { cn } from "../../lib/utils";
import {
  type CreateSecretRequest,
  type SecretKind,
  type UpdateSecretRequest,
} from "../../lib/secrets";
import { useSecretReferences } from "../../hooks/useSecretReferences";

export function SecretReferenceManager() {
  const {
    secrets,
    loading,
    saving,
    rotating,
    error,
    refresh,
    createSecret,
    removeSecret,
    rotateSecret,
  } = useSecretReferences();
  const form = useForm<CreateSecretRequest>({
    defaultValues: {
      label: "",
      kind: "acme_account_key",
      secret_value: "",
    },
  });
  const secretValue = form.watch("secret_value");
  const [rotateTarget, setRotateTarget] = useState<string | null>(null);
  const [rotateValue, setRotateValue] = useState("");
  const [rotateLabel, setRotateLabel] = useState("");

  const hasSecrets = useMemo(() => secrets.length > 0, [secrets]);

  function resetForm() {
    form.reset({
      label: "",
      kind: "acme_account_key",
      secret_value: "",
    });
  }

  async function handleCreate(values: CreateSecretRequest) {
    const created = await createSecret(values);
    if (created) {
      resetForm();
      setRotateTarget(null);
    }
    form.setValue("secret_value", "");
  }

  async function handleDelete(id: string) {
    await removeSecret(id);
  }

  async function handleRotate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!rotateTarget) return;
    const payload: UpdateSecretRequest = {
      id: rotateTarget,
      secret_value: rotateValue,
      label: rotateLabel || undefined,
    };
    const updated = await rotateSecret(payload);
    if (updated) {
      setRotateValue("");
      setRotateLabel("");
      setRotateTarget(null);
    }
    setRotateValue("");
  }

  function formatKind(kind: SecretKind) {
    switch (kind) {
      case "dns_provider_token":
        return "DNS provider token";
      case "dns_provider_access_key":
        return "DNS provider access key";
      case "dns_provider_secret_key":
        return "DNS provider secret key";
      case "acme_account_key":
        return "ACME account key";
      case "managed_private_key":
        return "Managed private key";
      default:
        return kind;
    }
  }

  function formatDate(iso: string) {
    const date = new Date(iso);
    return Number.isNaN(date.getTime())
      ? "—"
      : date.toLocaleString(undefined, {
          month: "short",
          day: "numeric",
          year: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        });
  }

  return (
    <div className="space-y-4">
      {error ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}

      <div className="grid gap-6 lg:grid-cols-[1.2fr,1fr]">
        <Card className="shadow-soft">
          <CardHeader className="flex-row items-start justify-between gap-3 space-y-0">
            <div className="flex items-center gap-3">
              <KeyRound className="h-5 w-5 text-primary" />
              <div>
                <CardTitle className="text-sm font-semibold">
                  Secret references
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  Friendly labels, created dates, and kinds. No secret bytes
                  ever leave Rust.
                </p>
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void refresh()}
              disabled={loading}
              className="gap-2"
            >
              <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
              Refresh
            </Button>
          </CardHeader>

          <CardContent className="space-y-3">
            {loading ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-4 text-sm text-muted-foreground">
                Loading secrets…
              </div>
            ) : null}
            {!loading && !hasSecrets ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-5 text-sm text-muted-foreground">
                No secret references yet. Add an ACME account or managed key
                to begin.
              </div>
            ) : null}
            {secrets.map((secret) => (
              <div
                key={secret.id}
                className="rounded-lg border bg-background/80 p-4 shadow-sm"
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-2 text-sm font-semibold">
                      {secret.label || "Untitled"}
                      <span className="text-xs font-medium text-muted-foreground">
                        {secret.id}
                      </span>
                    </div>
                    <div className="mt-1 text-sm text-muted-foreground">
                      {formatKind(secret.kind)} · Created {formatDate(secret.created_at)}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="gap-1"
                      onClick={() => {
                        setRotateTarget(secret.id);
                        setRotateValue("");
                        setRotateLabel(secret.label);
                      }}
                    >
                      <RefreshCw className="h-4 w-4" />
                      Replace
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="gap-1 text-destructive hover:bg-destructive/10"
                      onClick={() => void handleDelete(secret.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                      Remove
                    </Button>
                  </div>
                </div>

                {rotateTarget === secret.id ? (
                  <form className="mt-3 space-y-3" onSubmit={handleRotate}>
                    <div className="grid gap-3 sm:grid-cols-[2fr,1fr]">
                      <div>
                        <Label htmlFor={`secret-rotate-value-${secret.id}`}>
                          New secret value
                        </Label>
                        <Input
                          id={`secret-rotate-value-${secret.id}`}
                          type="password"
                          autoComplete="off"
                          required
                          placeholder="Paste token or key material (kept in Rust only)"
                          value={rotateValue}
                          onChange={(e) => setRotateValue(e.target.value)}
                        />
                      </div>
                      <div>
                        <Label htmlFor={`secret-rotate-label-${secret.id}`}>
                          Label (optional)
                        </Label>
                        <Input
                          id={`secret-rotate-label-${secret.id}`}
                          value={rotateLabel}
                          onChange={(e) => setRotateLabel(e.target.value)}
                        />
                        <p className="mt-2 text-xs text-muted-foreground">
                          Label helps identify the secret reference; the ID stays stable.
                        </p>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        type="submit"
                        size="sm"
                        disabled={rotating}
                        className="gap-2"
                      >
                        {rotating ? (
                          <RefreshCw className="h-4 w-4 animate-spin" />
                        ) : (
                          <RefreshCw className="h-4 w-4" />
                        )}
                        Save replacement
                      </Button>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => setRotateTarget(null)}
                      >
                        Cancel
                      </Button>
                    </div>
                  </form>
                ) : null}
              </div>
            ))}
          </CardContent>
        </Card>

        <Card className="shadow-soft">
          <CardHeader className="flex-row items-start gap-3 space-y-0">
            <Shield className="h-5 w-5 text-primary" />
            <div>
              <CardTitle className="text-sm font-semibold">
                Add secret reference
              </CardTitle>
              <p className="text-sm text-muted-foreground">
                UI sends the value into Rust once. Only metadata is stored for listing.
              </p>
            </div>
          </CardHeader>

          <CardContent>
            <Form {...form}>
              <form className="space-y-4" onSubmit={form.handleSubmit(handleCreate)}>
                <FormField
                  control={form.control}
                  name="label"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Label</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="e.g., Cloudflare prod DNS"
                          {...field}
                          required
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="kind"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Secret type</FormLabel>
                      <Select value={field.value} onValueChange={field.onChange}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="Select secret type" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="acme_account_key">ACME account key</SelectItem>
                          <SelectItem value="managed_private_key">Managed private key</SelectItem>
                        </SelectContent>
                      </Select>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="secret_value"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Secret value</FormLabel>
                      <FormControl>
                        <Input
                          type="password"
                          autoComplete="off"
                          placeholder="Paste token or key material. It is sent into Rust only once."
                          {...field}
                          required
                        />
                      </FormControl>
                      <FormDescription>
                        Value is never returned to the UI. A prefixed reference ID will be created.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <Button
                  type="submit"
                  className="w-full gap-2"
                  disabled={saving || !secretValue}
                >
                  {saving ? (
                    <RefreshCw className="h-4 w-4 animate-spin" />
                  ) : (
                    <Plus className="h-4 w-4" />
                  )}
                  Add secret reference
                </Button>
              </form>
            </Form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
