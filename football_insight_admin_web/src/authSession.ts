const SESSION_KEY = "football-insight-admin-web.session";
export class AuthSession {
  private token: string | null;
  constructor(
    private storage: Pick<Storage, "getItem" | "setItem" | "removeItem">,
  ) {
    try {
      this.token = storage.getItem(SESSION_KEY);
    } catch {
      this.token = null;
    }
  }
  getToken(): string | null {
    return this.token;
  }
  setToken(token: string): void {
    this.token = token;
    try {
      this.storage.setItem(SESSION_KEY, token);
    } catch {
      /* in-memory session still works */
    }
  }
  invalidate(requestToken: string): boolean {
    if (this.token !== requestToken) return false;
    this.token = null;
    try {
      this.storage.removeItem(SESSION_KEY);
    } catch {
      /* storage may be unavailable */
    }
    return true;
  }
}

export function browserSessionStorage(): Pick<
  Storage,
  "getItem" | "setItem" | "removeItem"
> {
  try {
    return window.localStorage;
  } catch {
    const memory = new Map<string, string>();
    return {
      getItem: (key) => memory.get(key) ?? null,
      setItem: (key, value) => {
        memory.set(key, value);
      },
      removeItem: (key) => {
        memory.delete(key);
      },
    };
  }
}
