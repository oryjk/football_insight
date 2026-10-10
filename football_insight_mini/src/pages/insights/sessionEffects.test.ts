import { describe, expect, test } from 'bun:test'
import { effectScope, nextTick, ref, watch } from 'vue'
import { createInsightsSessionSync } from './sessionEffects'
import type { CurrentUser } from '../../types/auth'

const userA: CurrentUser = {
  id: 'a',
  display_name: '用户a',
  account_identifier: 'account-a',
  avatar_url: null,
  has_wechat_binding: true,
  membership_tier: 'V3',
  created_at: '2026-01-01T00:00:00Z',
}

describe('insights session sync（恢复发布后刷新会员洞察）', () => {
  /** 与页面相同的接线，走真实 Vue watch。 */
  function wireSync(options: { underReview: boolean }) {
    const currentUser = ref<CurrentUser | null>(null)
    let insightLoads = 0
    const sync = createInsightsSessionSync({
      isUnderReview: () => options.underReview,
      loadInsights: async () => {
        insightLoads += 1
      },
    })
    const scope = effectScope()
    scope.run(() => {
      watch(currentUser, (user) => {
        sync.handleSessionUserPublished(user)
      })
    })
    return {
      currentUser,
      get insightLoads() {
        return insightLoads
      },
      stop: () => scope.stop(),
    }
  }

  test('后台恢复发布已认证用户：自动加载洞察，无需切 Tab', async () => {
    const fixture = wireSync({ underReview: false })

    fixture.currentUser.value = userA
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(fixture.insightLoads).toBe(1)
    fixture.stop()
  })

  test('退出清空用户：不加载洞察（模板已切游客视图）', async () => {
    const fixture = wireSync({ underReview: false })
    fixture.currentUser.value = userA
    await nextTick()

    fixture.currentUser.value = null
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(fixture.insightLoads).toBe(1)
    fixture.stop()
  })

  test('审核中即使发布用户也不拉取会员数据', async () => {
    const fixture = wireSync({ underReview: true })

    fixture.currentUser.value = userA
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(fixture.insightLoads).toBe(0)
    fixture.stop()
  })
})
