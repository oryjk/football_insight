<template>
  <view class="domestic-wrap">
    <view class="section-card">
      <view class="section-head" hover-class="section-pressed" @tap="domesticExpanded = !domesticExpanded">
        <view class="section-head__left">
          <text class="section-step">STEP 3</text>
          <text class="section-title">国内球员调整（选填）</text>
        </view>
        <text class="section-toggle">
          {{ summaryText }} · {{ domesticExpanded ? '收起' : '展开' }}
        </text>
      </view>

      <view v-if="domesticExpanded" class="domestic-body">
        <view class="sub-block">
          <view class="sub-block__head">
            <text class="sub-block__title">国内重点补强（{{ domesticIns.length }}/{{ maxDomesticIns }}）</text>
            <button
              class="mini-btn"
              :class="{ 'mini-btn--disabled': domesticIns.length >= maxDomesticIns }"
              hover-class="section-pressed"
              @tap="emit('open-domestic-picker')"
            >
              + 选人
            </button>
          </view>

          <view v-if="domesticIns.length" class="picked-tags">
            <view v-for="item in domesticIns" :key="item.key" class="picked-tag">
              <text class="picked-tag__text">
                {{ item.player_name }} · {{ directorPositionLabel(item.position) }}
              </text>
              <text class="picked-tag__close" @tap="emit('remove-domestic-in', item.key)">✕</text>
            </view>
          </view>
          <text v-else class="sub-hint">可选 0~{{ maxDomesticIns }} 名本土重点引援目标</text>
        </view>

        <view class="sub-block">
          <view class="sub-block__head">
            <text class="sub-block__title">本队国内离队建议（{{ domesticOuts.length }}/{{ maxDomesticOuts }}）</text>
          </view>
          <view v-if="squadDomestic.length" class="domestic-grid">
            <button
              v-for="player in squadDomestic"
              :key="player.player_id"
              class="domestic-pill"
              :class="{ 'domestic-pill--active': domesticOuts.includes(player.player_id) }"
              hover-class="section-pressed"
              @tap="emit('toggle-domestic-out', player.player_id)"
            >
              {{ player.player_name }}
            </button>
          </view>
          <text v-else class="sub-hint">暂无本队国内现役名单</text>
        </view>
      </view>
    </view>

    <view class="section-card">
      <view class="section-head">
        <view class="section-head__left">
          <text class="section-title">总监宣言（选填）</text>
        </view>
        <text class="char-count">{{ charCount }}/{{ DIRECTOR_MANIFESTO_MAX }}</text>
      </view>

      <textarea
        :value="manifesto"
        class="manifesto-input"
        :maxlength="DIRECTOR_MANIFESTO_MAX"
        placeholder="写一句你的建队宣言，发到微信群更有气势！"
        @input="onManifestoInput"
      />

      <view class="preset-list">
        <button
          v-for="preset in DIRECTOR_MANIFESTO_PRESETS"
          :key="preset"
          class="preset-chip"
          hover-class="section-pressed"
          @tap="emit('update:manifesto', preset)"
        >
          {{ preset }}
        </button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DirectorSquadPlayerDto, PlanPlayerItem } from '../../../types/director'
import { directorPositionLabel } from '../../../utils/directorLinks'
import {
  countUnicodeChars,
  DIRECTOR_MANIFESTO_MAX,
  DIRECTOR_MANIFESTO_PRESETS,
} from '../helpers'

const props = defineProps<{
  squadDomestic: DirectorSquadPlayerDto[]
  domesticIns: PlanPlayerItem[]
  domesticOuts: number[]
  maxDomesticIns: number
  maxDomesticOuts: number
  manifesto: string
}>()

const emit = defineEmits<{
  (e: 'open-domestic-picker'): void
  (e: 'remove-domestic-in', key: string): void
  (e: 'toggle-domestic-out', playerId: number): void
  (e: 'update:manifesto', value: string): void
}>()

const domesticExpanded = ref(false)
const charCount = computed(() => countUnicodeChars(props.manifesto))
const summaryText = computed(() => {
  const ins = props.domesticIns.length
  const outs = props.domesticOuts.length
  if (!ins && !outs) return '未设置'
  return `补强${ins} / 离队${outs}`
})

function onManifestoInput(event: unknown): void {
  const value = (event as { detail?: { value?: string } } | null)?.detail?.value
  emit('update:manifesto', typeof value === 'string' ? value : '')
}
</script>

<style scoped lang="css">
.domestic-wrap {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-20);
}

.section-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-head__left {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
}

.section-step {
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.section-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.section-toggle {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.domestic-body {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-20);
  padding-top: var(--fi-space-12);
  border-top: var(--fi-border-divider);
}

.sub-block {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.sub-block__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-10);
}

.sub-block__title {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.mini-btn {
  margin: 0;
  padding: var(--fi-space-8) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.mini-btn::after {
  border: 0;
}

.mini-btn--disabled {
  opacity: var(--fi-opacity-pressed);
}

.sub-hint {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.picked-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-10);
}

.picked-tag {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.picked-tag__text {
  line-height: var(--fi-leading-none);
}

.picked-tag__close {
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-none);
}

.domestic-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-10);
}

.domestic-pill {
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.domestic-pill::after {
  border: 0;
}

.domestic-pill--active {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
  border-color: var(--fi-color-primary);
  font-weight: var(--fi-weight-bold);
}

.char-count {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-variant-numeric: tabular-nums;
}

.manifesto-input {
  width: 100%;
  height: calc(var(--fi-space-64) * 2);
  box-sizing: border-box;
  padding: var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-normal);
}

.preset-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-10);
}

.preset-chip {
  margin: 0;
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
}

.preset-chip::after {
  border: 0;
}

.section-pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
