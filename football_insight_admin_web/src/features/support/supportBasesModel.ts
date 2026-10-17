import type { SupportBaseMatch } from "../../api";

export const MAX_SUPPORT_BASE = 100000;

// Empty or non-integer input is invalid; the save button stays disabled for it.
export function parseBase(value: string): number | null {
  const text = value.trim();
  if (!/^\d{1,6}$/.test(text)) return null;
  const parsed = Number(text);
  return parsed <= MAX_SUPPORT_BASE ? parsed : null;
}

export function displayedCount(real: number, baseInput: string): number | null {
  const base = parseBase(baseInput);
  return base === null ? null : real + base;
}

export function isChanged(
  match: SupportBaseMatch,
  homeInput: string,
  awayInput: string,
): boolean {
  return (
    parseBase(homeInput) !== match.home_team.base ||
    parseBase(awayInput) !== match.away_team.base
  );
}

export function canSave(
  match: SupportBaseMatch,
  homeInput: string,
  awayInput: string,
): boolean {
  return (
    parseBase(homeInput) !== null &&
    parseBase(awayInput) !== null &&
    isChanged(match, homeInput, awayInput)
  );
}

export function matchTitle(match: SupportBaseMatch): string {
  return `第 ${match.round_number} 轮 · ${match.match_date} ${match.match_time}`;
}
