<template>
  <view class="dot" :class="{ 'dot--selected': selected, 'dot--empty': !player.player, 'dot--compact': compact }">
    <view class="dot__face">
      <template v-if="player.player">
        <view v-if="player.player.avatar_url" class="dot__avatar"><FiAvatar :src="player.player.avatar_url" :size="96" :name="player.player.name" /></view>
        <text v-else class="dot__fallback">{{ player.player.shirt_number ?? player.player.name.slice(0, 1) }}</text>
        <text v-if="player.player.shirt_number !== null && player.player.avatar_url" class="dot__number">
          {{ player.player.shirt_number }}
        </text>
        <text v-if="player.player.former" class="dot__former">前</text>
        <text v-if="exclusive" class="dot__exclusive">独家</text>
      </template>
      <text v-else class="dot__plus">+</text>
    </view>
    <text class="dot__name">{{ player.player ? player.player.name : player.label }}</text>
  </view>
</template>

<script setup lang="ts">
import FiAvatar from '../../../components/FiAvatar.vue'
import type { PitchPlayer } from '../lineupViewModel'

defineProps<{
  player: PitchPlayer
  selected?: boolean
  /** 缩小版，供分享预览复用。 */
  compact?: boolean
  /** 不在共识阵容里的"独家"球员标记。 */
  exclusive?: boolean
}>()
</script>

<style scoped>
.dot {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-4);
  width: calc(var(--fi-component-lineup-dot-size) + var(--fi-space-32));
}

.dot__face {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--fi-component-lineup-dot-size);
  height: var(--fi-component-lineup-dot-size);
  border: var(--fi-border-width) solid var(--fi-primitive-white);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-soft);
}

.dot--selected .dot__face {
  border-color: var(--fi-color-primary);
}

.dot--empty .dot__face {
  border-style: dashed;
  background: rgba(var(--fi-primitive-white-rgb), 0.22);
  box-shadow: none;
}

.dot__avatar {
  overflow: hidden;
  width: 100%;
  height: 100%;
  border-radius: var(--fi-radius-round);
}

.dot__fallback {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
}

.dot__plus {
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.dot__number {
  position: absolute;
  right: calc(-1 * var(--fi-space-6));
  bottom: calc(-1 * var(--fi-space-6));
  min-width: var(--fi-space-32);
  padding: var(--fi-space-2) var(--fi-space-6);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  text-align: center;
}

.dot__former {
  position: absolute;
  top: calc(-1 * var(--fi-space-6));
  left: calc(-1 * var(--fi-space-6));
  padding: var(--fi-space-2) var(--fi-space-8);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.dot__exclusive {
  position: absolute;
  top: calc(-1 * var(--fi-space-6));
  right: calc(-1 * var(--fi-space-14));
  padding: var(--fi-space-2) var(--fi-space-8);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-strong);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.dot__name {
  max-width: 100%;
  overflow: hidden;
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dot--empty .dot__name {
  color: rgba(var(--fi-primitive-white-rgb), 0.72);
  font-weight: var(--fi-weight-bold);
}
</style>
