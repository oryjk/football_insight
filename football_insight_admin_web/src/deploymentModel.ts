export function adminApiBaseForMount(basePath: string): string {
  return `${basePath.replace(/\/+$/, "")}/api/v1/admin`;
}
