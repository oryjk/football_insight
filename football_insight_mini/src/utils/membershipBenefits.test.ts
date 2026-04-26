import { describe, expect, test } from 'bun:test'

import { resolveMembershipBenefitsLocked } from './membershipBenefits'

describe('resolveMembershipBenefitsLocked', () => {
  test('keeps benefits unlocked when the backend field is missing', () => {
    expect(resolveMembershipBenefitsLocked(null)).toBe(false)
    expect(resolveMembershipBenefitsLocked({})).toBe(false)
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: undefined })).toBe(false)
  })

  test('locks benefits only when the backend explicitly returns false', () => {
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: true })).toBe(false)
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: false })).toBe(true)
  })
})
