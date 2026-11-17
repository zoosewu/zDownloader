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

/** Links in something dropped on the page: links dragged from a browser, or text files. */
export async function linksDropped(data: DataTransfer): Promise<string[]> {
  const texts = await Promise.all([...data.files].filter((f) => /\.txt$/i.test(f.name)).map((f) => f.text()));
  texts.push(data.getData('text/uri-list'), data.getData('text/plain'));
  return [...new Set(linksIn(texts.join('\n')))];
}
