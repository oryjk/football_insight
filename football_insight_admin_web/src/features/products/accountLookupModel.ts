export interface AccountLookupResult<T> {
  id: string;
  value?: T;
  error?: unknown;
}
export async function lookupAccountMetadata<T>(
  ids: string[],
  load: (id: string, signal: AbortSignal) => Promise<T>,
  signal: AbortSignal,
): Promise<AccountLookupResult<T>[]> {
  const unique = [...new Set(ids)];
  const results: Array<AccountLookupResult<T> | undefined> = Array(
    unique.length,
  );
  let next = 0;
  await Promise.all(
    Array.from({ length: Math.min(4, unique.length) }, async () => {
      while (!signal.aborted && next < unique.length) {
        const index = next++;
        const id = unique[index];
        try {
          const value = await load(id, signal);
          if (!signal.aborted) results[index] = { id, value };
        } catch (error) {
          if (!signal.aborted) results[index] = { id, error };
        }
      }
    }),
  );
  return results.filter(
    (value): value is AccountLookupResult<T> => value !== undefined,
  );
}
