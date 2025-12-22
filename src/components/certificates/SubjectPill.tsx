export function SubjectPill({ text }: { text: string }) {
  return (
    <span className="rounded-full bg-muted px-3 py-1 text-xs font-semibold text-foreground">
      {text}
    </span>
  );
}
