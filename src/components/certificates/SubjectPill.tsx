export function SubjectPill({ text }: { text: string }) {
  return (
    <span className="rounded-full bg-muted px-3 py-1 text-s font-semibold text-foreground">
      {text}
    </span>
  );
}
