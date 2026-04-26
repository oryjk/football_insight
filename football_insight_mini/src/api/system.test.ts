import { describe, expect, test } from 'bun:test'

import { buildSystemConfigUrl } from './system'

describe('buildSystemConfigUrl', () => {
  test('uses the generic system_config endpoint with the mini program version', () => {
    expect(buildSystemConfigUrl('1.0.0')).toBe('/system_config?version=1.0.0')
  })

  test('encodes app id when it is provided', () => {
    expect(buildSystemConfigUrl('1.0.0 beta', 'wx-test-app')).toBe(
      '/system_config?version=1.0.0+beta&app_id=wx-test-app',
    )
  })
})
