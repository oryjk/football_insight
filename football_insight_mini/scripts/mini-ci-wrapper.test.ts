import { readFileSync } from 'node:fs'

import { describe, expect, test } from 'bun:test'

const wrapperSource = readFileSync(new URL('./mini-ci.mjs', import.meta.url), 'utf8')

describe('football mini ci wrapper', () => {
  test('uses the same review upload flow as registration system with football project identity', () => {
    expect(wrapperSource.includes('const reviewInternalToken = "football_insight_mini";')).toBe(true)
    expect(wrapperSource.includes('|| "football_insight_mini";')).toBe(true)
    expect(wrapperSource.includes('/api/admin/reviews')).toBe(true)
    expect(wrapperSource.includes('"X-Internal-Token": reviewInternalToken')).toBe(true)
    expect(wrapperSource.includes('project_code: reviewProjectCode')).toBe(true)
    expect(wrapperSource.includes('remark: "mp:upload 自动创建"')).toBe(true)
  })

  test('forwards the resolved upload version to shared ci only once', () => {
    expect(wrapperSource.includes('const uploadVersion = command === "upload" ? resolveUploadVersion() : null;')).toBe(true)
    expect(wrapperSource.includes('[...extraArgs, "--version", uploadVersion]')).toBe(true)
  })

  test('syncs manifest version before running the shared ci command', () => {
    expect(wrapperSource.includes('import { syncManifestVersion } from "./sync-manifest-version.mjs";')).toBe(true)
    expect(wrapperSource.includes('syncManifestVersion();')).toBe(true)
  })
})
