/** Textareas normalize CRLF. Preserve original endings outside the edited range. */
export function updateTextDraft(source: string, value: string): string {
  const normalized = source.replace(/\r\n/g, "\n");
  if (normalized === value) return source;
  let start = 0;
  while (
    start < normalized.length &&
    start < value.length &&
    normalized[start] === value[start]
  )
    start++;
  let end = normalized.length;
  let nextEnd = value.length;
  while (
    end > start &&
    nextEnd > start &&
    normalized[end - 1] === value[nextEnd - 1]
  ) {
    end--;
    nextEnd--;
  }
  const sourceOffset = (offset: number) => {
    let raw = 0;
    for (let i = 0; i < offset; i++, raw++) {
      if (source[raw] === "\r" && source[raw + 1] === "\n") raw++;
    }
    return raw;
  };
  const ending = source.includes("\r\n") ? "\r\n" : "\n";
  return (
    source.slice(0, sourceOffset(start)) +
    value.slice(start, nextEnd).replace(/\n/g, ending) +
    source.slice(sourceOffset(end))
  );
}
