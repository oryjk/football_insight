// Mirrors the backend's MAX_BRAND_SUBTITLE_CHARS; the server revalidates on save.
export const MAX_SUBTITLE_CHARS = 20;
// The mini program's built-in copy, shown when the subtitle is left blank.
export const DEFAULT_BRAND_SUBTITLE = "最懂球迷的工具";

export function validateSubtitle(value: string): string | null {
  const text = value.trim();
  if (/[\u0000-\u001f\u007f-\u009f]/.test(text)) return "不能包含换行或控制字符";
  if ([...text].length > MAX_SUBTITLE_CHARS)
    return `最多 ${MAX_SUBTITLE_CHARS} 个字`;
  return null;
}

export function canSaveSubtitle(input: string, saved: string): boolean {
  return validateSubtitle(input) === null && input.trim() !== saved.trim();
}
