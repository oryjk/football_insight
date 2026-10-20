import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { CurrentUser } from '../types/auth'
import { createOfficialBindingGate } from './officialBindingGate'
import { resolveMembershipBenefitsLocked } from '../utils/membershipBenefits'

function setup(required: boolean | null = true) {
  const currentUser = ref<CurrentUser | null>({
    id: 'member-a', account_identifier: 'member-a', display_name: 'A', avatar_url: null,
    has_wechat_binding: true, membership_tier: 'V1', created_at: '',
    official_wechat_bound: false, official_wechat_binding_required: required ?? undefined,
    membership_benefits_enabled: true,
  })
  let refreshes = 0
  const session = {
    currentUser, hasAccessToken: ref(true), revision: ref(1),
    ensureSessionReady: async () => {
      refreshes += 1
      if (currentUser.value) currentUser.value = { ...currentUser.value }
      return currentUser.value
    },
  }
  const ready = ref(false), review = ref(false)
  const gate = createOfficialBindingGate(session, review, ready)
  return { session, ready, review, gate, refreshes: () => refreshes }
}

describe('official binding gate', () => {
  test('refreshes a bound account on return so unsubscribe locks and re-follow restores benefits', async () => {
    const { gate, ready, session } = setup(false)
    let serverUser = { ...session.currentUser.value!, official_wechat_bound: true, official_wechat_subscribed: true }
    session.currentUser.value = serverUser
    session.ensureSessionReady = async () => {
      session.currentUser.value = { ...serverUser }
      return session.currentUser.value
    }
    try {
      ready.value = true
      await gate.handlePageShow()
      gate.handlePageHide()
      serverUser = { ...serverUser, official_wechat_subscribed: false, membership_benefits_enabled: false }
      await gate.handlePageShow()
      expect(session.currentUser.value?.official_wechat_subscribed).toBe(false)
      expect(resolveMembershipBenefitsLocked(session.currentUser.value)).toBe(true)
      expect(gate.visible.value).toBe(false)
      gate.handlePageHide()
      serverUser = { ...serverUser, official_wechat_subscribed: true, membership_benefits_enabled: true }
      await gate.handlePageShow()
      expect(resolveMembershipBenefitsLocked(session.currentUser.value)).toBe(false)
    } finally { gate.dispose() }
  })
  test('waits for confirmed page state, prompts once per visit, and allows manual reopening', async () => {
    const { gate, ready, session, refreshes } = setup()
    try {
      await gate.handlePageShow()
      expect(gate.visible.value).toBe(false)
      ready.value = true
      expect(gate.visible.value).toBe(true)
      await gate.close()
      expect(refreshes()).toBe(2)
      expect(gate.visible.value).toBe(false)
      session.currentUser.value = { ...session.currentUser.value! }
      expect(gate.visible.value).toBe(false)
      gate.open()
      expect(gate.visible.value).toBe(true)
      await gate.close()
      gate.handlePageHide()
      await gate.handlePageShow()
      expect(gate.visible.value).toBe(true)
    } finally { gate.dispose() }
  })

  test('does not prompt paid exempt users or guests; a different non-paying account gets its own prompt', async () => {
    const { gate, ready, session } = setup(false)
    try {
      ready.value = true
      await gate.handlePageShow()
      expect(gate.visible.value).toBe(false)
      session.currentUser.value = null
      session.hasAccessToken.value = false
      expect(gate.visible.value).toBe(false)
      session.hasAccessToken.value = true
      session.currentUser.value = {
        id: 'member-b', account_identifier: 'b', display_name: 'B', avatar_url: null,
        has_wechat_binding: true, membership_tier: 'V9', created_at: '',
        official_wechat_bound: false, official_wechat_binding_required: true,
      }
      expect(gate.visible.value).toBe(true)
    } finally { gate.dispose() }
  })

  test('review and hidden pages do not prompt; server-confirmed binding closes the sheet', async () => {
    const { gate, ready, review, session } = setup()
    try {
      ready.value = true
      review.value = true
      await gate.handlePageShow()
      expect(gate.visible.value).toBe(false)
      gate.handlePageHide()
      review.value = false
      expect(gate.visible.value).toBe(false)
      await gate.handlePageShow()
      expect(gate.visible.value).toBe(true)
      session.currentUser.value = { ...session.currentUser.value!, official_wechat_bound: true, official_wechat_binding_required: false }
      expect(gate.visible.value).toBe(false)
    } finally { gate.dispose() }
  })

  test('refreshes missing login enrichment before deciding whether to prompt', async () => {
    const { gate, ready, session } = setup(null)
    try {
      ready.value = true
      session.ensureSessionReady = async () => {
        session.currentUser.value = { ...session.currentUser.value!, official_wechat_binding_required: false }
        return session.currentUser.value
      }
      await gate.handlePageShow()
      expect(gate.visible.value).toBe(false)
      expect(session.currentUser.value?.official_wechat_binding_required).toBe(false)
    } finally { gate.dispose() }
  })
})
