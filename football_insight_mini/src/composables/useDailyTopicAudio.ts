import { computed, onScopeDispose, ref, watch, type Ref } from 'vue'
import { onHide, onUnload } from '@dcloudio/uni-app'
import type { DailyTopicEdition } from '../types/dailyTopic'
import { topicPlaybackSignature } from '../utils/dailyTopics'
import { createTopicAudioClock } from '../utils/topicAudioClock'

const cachedAudioPath = new Map<string, string>()
const inflightPreload = new Map<string, Promise<string | null>>()

function preloadTopicAudio(url: string): Promise<string | null> {
  // #ifdef MP-WEIXIN
  const cached = cachedAudioPath.get(url)
  if (cached) return Promise.resolve(cached)
  const pending = inflightPreload.get(url)
  if (pending) return pending
  if (typeof uni?.downloadFile !== 'function') return Promise.resolve(null)
  const task = new Promise<string | null>((resolve) => {
    try {
      uni.downloadFile({
        url,
        success: (res) => {
          const path = res.statusCode === 200 && typeof res.tempFilePath === 'string' && res.tempFilePath
            ? res.tempFilePath
            : null
          if (path) cachedAudioPath.set(url, path)
          resolve(path)
        },
        fail: () => resolve(null),
      })
    } catch {
      resolve(null)
    }
  }).finally(() => {
    inflightPreload.delete(url)
  })
  inflightPreload.set(url, task)
  return task
  // #endif
  // #ifndef MP-WEIXIN
  void url
  return Promise.resolve(null)
  // #endif
}

export function useDailyTopicAudio(edition: Ref<DailyTopicEdition>) {
  const activeId = ref<string | null>(null)
  const status = ref<'idle' | 'loading' | 'playing' | 'paused' | 'error'>('idle')
  const currentTime = ref(0)
  const duration = ref(0)
  const errorMessage = ref('')
  const clock = createTopicAudioClock()
  let playingSignature = ''
  let context: UniApp.InnerAudioContext | null = null
  let preloadTimer: ReturnType<typeof setTimeout> | null = null
  const availableIds = computed(() => edition.value.items
    .filter(item => topicPlaybackSignature(edition.value, item.id)).map(item => item.id))
  const activeTitle = computed(() => edition.value.items.find(item => item.id === activeId.value)?.title ?? '')

  function clearPreloadTimer() {
    if (preloadTimer !== null) {
      clearTimeout(preloadTimer)
      preloadTimer = null
    }
  }

  function scheduleLeadPreload() {
    clearPreloadTimer()
    const firstId = availableIds.value[0]
    const firstUrl = firstId ? edition.value.items.find(item => item.id === firstId)?.audio.url : null
    if (!firstUrl || cachedAudioPath.has(firstUrl) || inflightPreload.has(firstUrl)) return
    preloadTimer = setTimeout(() => {
      preloadTimer = null
      void preloadTopicAudio(firstUrl)
    }, 600)
  }

  function stop() {
    const previous = context
    context = null
    previous?.destroy()
    activeId.value = null
    playingSignature = ''
    status.value = 'idle'
    currentTime.value = 0
    clock.reset()
    duration.value = 0
    errorMessage.value = ''
  }

  function startAudio(audio: UniApp.InnerAudioContext, remoteUrl: string, localPath: string | null) {
    let usedLocal = Boolean(localPath)
    const isCurrent = () => context === audio
    audio.onPlay(() => { if (isCurrent()) { status.value = 'playing'; clock.setPlaying(true) } })
    audio.onPause(() => { if (isCurrent()) { status.value = 'paused'; clock.setPlaying(false) } })
    audio.onWaiting(() => {
      if (!isCurrent()) return
      clock.setPlaying(false)
      if (!audio.paused && status.value !== 'error') status.value = 'loading'
    })
    audio.onCanplay(() => {
      if (!isCurrent()) return
      duration.value = Number.isFinite(audio.duration) ? audio.duration : 0
      if (!audio.paused && status.value === 'loading') { status.value = 'playing'; clock.setPlaying(true) }
    })
    audio.onTimeUpdate(() => {
      if (!isCurrent()) return
      clock.setPlaying(!audio.paused && status.value === 'playing')
      const nextTime = clock.observe(audio.currentTime)
      if (nextTime > currentTime.value && !audio.paused && status.value === 'loading') { status.value = 'playing'; clock.setPlaying(true) }
      currentTime.value = nextTime
      duration.value = Number.isFinite(audio.duration) ? audio.duration : 0
    })
    audio.onSeeked(() => {
      if (isCurrent()) currentTime.value = clock.observe(audio.currentTime)
    })
    audio.onEnded(() => {
      if (!isCurrent()) return
      status.value = 'paused'
      clock.setPlaying(false)
      clock.seek(0, duration.value)
      currentTime.value = 0
      audio.seek(0)
    })
    audio.onError(() => {
      if (!isCurrent()) return
      // 若系统回收了临时文件，清掉缓存并自动回退到远端 URL 重试一次。
      if (usedLocal) {
        usedLocal = false
        cachedAudioPath.delete(remoteUrl)
        status.value = 'loading'
        audio.src = remoteUrl
        audio.play()
        return
      }
      status.value = 'error'
      clock.setPlaying(false)
      errorMessage.value = '音频暂时无法播放，点击重试'
    })
    audio.src = localPath || remoteUrl
    audio.play()
  }

  function toggle(id: string) {
    if (!availableIds.value.includes(id)) return
    if (id === activeId.value && context && status.value !== 'error') {
      if (status.value === 'loading' && !context.src) {
        stop()
      } else if (status.value === 'playing' || status.value === 'loading') {
        context.pause()
      } else {
        context.play()
      }
      return
    }
    const item = edition.value.items.find(item => item.id === id)
    const signature = topicPlaybackSignature(edition.value, id)
    if (!item?.audio.url || !signature) return
    const remoteUrl = item.audio.url
    stop()
    activeId.value = id
    playingSignature = signature
    status.value = 'loading'
    try {
      const audio = uni.createInnerAudioContext()
      context = audio
      const cached = cachedAudioPath.get(remoteUrl) ?? null
      if (cached) {
        startAudio(audio, remoteUrl, cached)
        return
      }
      const inflight = inflightPreload.get(remoteUrl)
      if (inflight) {
        void inflight.then((localPath) => {
          if (context !== audio || status.value !== 'loading') return
          startAudio(audio, remoteUrl, localPath)
        })
        return
      }
      startAudio(audio, remoteUrl, null)
      // 边播边在后台缓存，下次切换回来可秒开。
      void preloadTopicAudio(remoteUrl)
    } catch {
      status.value = 'error'
      errorMessage.value = '音频暂时无法播放，点击重试'
    }
  }

  function seek(seconds: number) {
    if (!context) return
    const previous = currentTime.value
    clock.setPlaying(!context.paused && status.value === 'playing')
    const target = clock.seek(seconds, duration.value)
    if (target === null) return
    currentTime.value = target
    try {
      context.seek(target)
      errorMessage.value = ''
    } catch {
      clock.reset(previous)
      currentTime.value = previous
      errorMessage.value = '暂时无法跳转播放位置，请重试'
    }
  }

  watch(edition, () => {
    scheduleLeadPreload()
    if (!activeId.value) return
    if (topicPlaybackSignature(edition.value, activeId.value) !== playingSignature) stop()
  }, { immediate: true })
  onHide(() => context?.pause())
  onUnload(() => { clearPreloadTimer(); stop() })
  onScopeDispose(() => { clearPreloadTimer(); stop() })

  return { activeId, activeTitle, status, currentTime, duration, errorMessage, availableIds, toggle, seek, stop }
}
