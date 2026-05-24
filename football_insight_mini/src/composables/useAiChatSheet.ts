import { ref } from 'vue'
import { getCurrentUser } from '../api/auth'
import { getPublicSystemConfig } from '../api/system'
import type { CurrentUser } from '../types/auth'
import type { PublicSystemConfig } from '../types/system'
import { getAccessToken, setAccessToken } from '../utils/authStorage'

export function useAiChatSheet() {
  const aiChatVisible = ref(false)
  const currentAiUser = ref<CurrentUser | null>(null)
  const aiPublicConfig = ref<PublicSystemConfig | null>(null)
  let publicConfigLoaded = false

  async function loadAiPublicConfig(): Promise<void> {
    if (publicConfigLoaded) {
      return
    }

    try {
      aiPublicConfig.value = await getPublicSystemConfig()
    } catch {
      aiPublicConfig.value = null
    } finally {
      publicConfigLoaded = true
    }
  }

  async function ensureAiUser(): Promise<CurrentUser | null> {
    if (!getAccessToken()) {
      currentAiUser.value = null
      return null
    }

    if (currentAiUser.value) {
      return currentAiUser.value
    }

    try {
      currentAiUser.value = await getCurrentUser()
    } catch {
      currentAiUser.value = null
    }

    if (!currentAiUser.value) {
      setAccessToken(null)
    }

    return currentAiUser.value
  }

  function promptAiLogin(): void {
    uni.showModal({
      title: '先登录再聊天',
      content: '登录后才可以和小罗继续对话，现在去“我的”页登录吗？',
      confirmText: '去登录',
      success: ({ confirm }) => {
        if (!confirm) {
          return
        }

        uni.switchTab({
          url: '/pages/user/index',
        })
      },
    })
  }

  async function openAiChat(): Promise<void> {
    const [user] = await Promise.all([
      ensureAiUser(),
      loadAiPublicConfig(),
    ])

    if (!user) {
      promptAiLogin()
      return
    }

    aiChatVisible.value = true
  }

  function closeAiChat(): void {
    aiChatVisible.value = false
  }

  return {
    aiChatVisible,
    currentAiUser,
    aiPublicConfig,
    openAiChat,
    closeAiChat,
  }
}
