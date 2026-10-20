<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="公众号关联"
    title="绑定公众号"
    fit-content
    :z-index="80"
    respect-window-bottom
    @update:visible="emit('close')"
  >
    <view class="official-bind">
      <text class="official-bind__summary">绑定一次，之后可在公众号里查询下一场比赛 ID。</text>
      <view v-if="code" class="official-bind__code-card">
        <text class="official-bind__code">{{ code }}</text>
        <text class="official-bind__caption">{{ remainingSeconds > 0 ? `剩余 ${countdown}` : '绑定码已过期，请重新获取' }}</text>
        <button class="official-bind__copy" :disabled="remainingSeconds <= 0 || loading" @tap="emit('copy')">复制绑定指令</button>
      </view>
      <view v-if="code && remainingSeconds > 0" class="official-bind__steps">
        <text>1. 复制绑定指令，打开「<text class="official-bind__account-link" @tap="emit('open-official')">何止编程</text>」公众号。</text>
        <text>2. 回复：绑定 {{ code }}</text>
        <text>3. 收到绑定成功回复后，返回小程序即可。</text>
      </view>
      <text v-else class="official-bind__caption">获取绑定码后，打开「<text class="official-bind__account-link" @tap="emit('open-official')">何止编程</text>」公众号，发送“绑定 + 绑定码”。</text>
      <text v-if="error" class="official-bind__error">{{ error }}</text>
      <text class="official-bind__warning">绑定后请保持关注公众号，取关后会员权益会暂停。</text>
    </view>
    <template #footer>
      <button class="official-bind__action" :disabled="loading" @tap="emit('get-code')">
        {{ loading ? '获取中…' : code ? '重新获取' : '获取绑定码' }}
      </button>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import { formatOfficialWechatBindCountdown } from '../helpers'

const props = defineProps<{
  visible: boolean
  code: string
  remainingSeconds: number
  loading: boolean
  error: string
}>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'get-code'): void
  (e: 'copy'): void
  (e: 'open-official'): void
}>()
const countdown = computed(() => formatOfficialWechatBindCountdown(props.remainingSeconds))
</script>

<style scoped>
.official-bind,
.official-bind__steps,
.official-bind__code-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
}

.official-bind__summary,
.official-bind__steps {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.official-bind__code-card {
  align-items: center;
  padding: var(--fi-space-24);
  border: var(--fi-border-card);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.official-bind__code {
  color: var(--fi-color-text-strong);
  font-family: monospace;
  font-size: var(--fi-font-50);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
  letter-spacing: var(--fi-space-6);
}

.official-bind__caption,
.official-bind__warning,
.official-bind__error {
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-loose);
  color: var(--fi-color-text-muted);
}

.official-bind__warning {
  padding: var(--fi-space-16);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
}

.official-bind__account-link {
  color: var(--fi-color-primary);
  text-decoration: underline;
}

.official-bind__error {
  color: var(--fi-color-primary);
}

.official-bind__copy,
.official-bind__action {
  margin: 0;
  padding: var(--fi-space-18) var(--fi-space-24);
  border: var(--fi-border-card);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.official-bind__action {
  width: 100%;
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
}

.official-bind__copy::after,
.official-bind__action::after {
  border: none;
}

.official-bind__copy[disabled],
.official-bind__action[disabled] {
  background: var(--fi-color-page);
  color: var(--fi-color-text-muted);
}
</style>
