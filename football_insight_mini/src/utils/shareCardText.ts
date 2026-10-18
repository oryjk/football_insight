export function wrapShareCardText(
  text: string, measure: (text: string) => number, width: number, maxLines: number,
): string[] {
  if (width <= 0 || maxLines <= 0) return []
  const lines: string[] = []
  let line = ''
  for (const character of Array.from(text.replace(/\s+/g, ' ').trim())) {
    if (!line || measure(line + character) <= width) { line += character; continue }
    lines.push(line.trim())
    if (lines.length === maxLines) {
      const last = Array.from(lines[lines.length - 1])
      while (last.length && measure(last.join('') + '…') > width) last.pop()
      lines[lines.length - 1] = last.join('') + '…'
      return lines
    }
    line = character.trimStart()
  }
  if (line) lines.push(line.trim())
  return lines
}
