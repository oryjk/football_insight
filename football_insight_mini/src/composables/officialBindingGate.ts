import { computed, ref, watch, type Ref } from 'vue'
import type { AppSession } from '../stores/appSession'
import { createUserOfficialBindFlow } from './officialBindFlow'

/** Page-owned binding flow: one automatic prompt per visible visit and account. */
export function createOfficialBindingGate(
  session: Pick<AppSession, 'currentUser' | 'hasAccessToken' | 'revision' | 'ensureSessionReady'>,
  underReview: Readonly<Ref<boolean>>,
  ready: Readonly<Ref<boolean>>,
) {
  const flow = createUserOfficialBindFlow(session, underReview)
  const active = ref(false)
  const checking = ref(false)
  const actor = computed(() => `${session.revision.value}:${session.currentUser.value?.id ?? ''}`)
  let promptedActor = ''
  let visit = 0
  let timer: ReturnType<typeof setInterval> | null = null

  const stopPrompt = watch([actor, session.currentUser, ready, underReview, active, checking], () => {
    if (!active.value || checking.value || !ready.value || underReview.value
      || session.currentUser.value?.official_wechat_binding_required !== true
      || !flow.canOpen.value || promptedActor === actor.value) return
    promptedActor = actor.value
    if (!flow.visible.value) flow.open()
  }, { flush: 'sync' })

  const stopTimer = watch([flow.visible, active], () => {
    if (timer !== null) clearInterval(timer)
    timer = null
    flow.refreshCountdown()
    if (active.value && flow.visible.value) timer = setInterval(flow.refreshCountdown, 1000)
  }, { flush: 'sync' })

  async function handlePageShow(): Promise<void> {
    const shown = ++visit
    checking.value = true
    active.value = true
    try {
      await flow.handlePageShow()
      // Login responses omit /auth/me enrichment; bypass its success cache before prompting.
      if (shown === visit && session.currentUser.value
        && session.currentUser.value.official_wechat_binding_required === undefined) {
        await session.ensureSessionReady({ force: true })
      }
    } catch {
      // appSession owns failure and session handling; no prompt without confirmed server state.
    } finally {
      if (shown === visit) checking.value = false
    }
  }

  function handlePageHide(): void {
    visit += 1
    active.value = false
    promptedActor = ''
    flow.handlePageHide()
  }

  function dispose(): void {
    handlePageHide()
    stopPrompt()
    stopTimer()
    if (timer !== null) clearInterval(timer)
    flow.dispose()
  }

  return { ...flow, handlePageShow, handlePageHide, dispose }
}
