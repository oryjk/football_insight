import { computed, onScopeDispose, ref, watch, type Ref } from 'vue'
import { onHide, onUnload } from '@dcloudio/uni-app'
import type { DailyTopicEdition } from '../types/dailyTopic'
import { topicPlaybackSignature } from '../utils/dailyTopics'
import { createTopicAudioClock } from '../utils/topicAudioClock'

export function useDailyTopicAudio(edition: Ref<DailyTopicEdition>) {
  const activeId = ref<string | null>(null)
  const status = ref<'idle' | 'loading' | 'playing' | 'paused' | 'error'>('idle')
  const currentTime = ref(0)
  const duration = ref(0)
  const errorMessage = ref('')
  const clock = createTopicAudioClock()
  let playingSignature = ''
  let context: UniApp.InnerAudioContext | null = null
  const availableIds = computed(() => edition.value.items
    .filter(item => topicPlaybackSignature(edition.value, item.id)).map(item => item.id))
  const activeTitle = computed(() => edition.value.items.find(item => item.id === activeId.value)?.title ?? '')

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

  function toggle(id: string) {
    if (!availableIds.value.includes(id)) return
    if (id === activeId.value && context && status.value !== 'error') {
      if (status.value === 'playing' || status.value === 'loading') context.pause()
      else context.play()
      return
    }
    const item = edition.value.items.find(item => item.id === id)
    const signature = topicPlaybackSignature(edition.value, id)
    if (!item?.audio.url || !signature) return
    stop()
    activeId.value = id
    playingSignature = signature
    status.value = 'loading'
    try {
      const audio = uni.createInnerAudioContext()
      context = audio
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
        status.value = 'error'
        clock.setPlaying(false)
        errorMessage.value = '音频暂时无法播放，点击重试'
      })
      audio.src = item.audio.url
      audio.play()
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
    if (!activeId.value) return
    if (topicPlaybackSignature(edition.value, activeId.value) !== playingSignature) stop()
  })
  onHide(() => context?.pause())
  onUnload(stop)
  onScopeDispose(stop)

  return { activeId, activeTitle, status, currentTime, duration, errorMessage, availableIds, toggle, seek, stop }
}
