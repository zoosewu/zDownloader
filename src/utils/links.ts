/** http(s) links in pasted or dropped text: one per line (spaces also separate). */
export function linksIn(text: string): string[] {
  return text
    .split(/\s+/)
    .map((s) => s.trim())
    .filter((s) => {
      try {
        const u = new URL(s);
        return (u.protocol === 'http:' || u.protocol === 'https:') && !!u.hostname;
      } catch {
        return false;
      }
    });
}
