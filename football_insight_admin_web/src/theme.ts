export type Theme = "dark" | "light";
const key = "football-insight-admin-web.theme";
export function readTheme(): Theme {
  try {
    return localStorage.getItem(key) === "light" ? "light" : "dark";
  } catch {
    return "dark";
  }
}
export function applyTheme(theme: Theme): void {
  document.documentElement.classList.toggle("light", theme === "light");
  document.documentElement.classList.toggle("dark", theme === "dark");
  try {
    localStorage.setItem(key, theme);
  } catch {
    /* theme works without persistent storage */
  }
}
