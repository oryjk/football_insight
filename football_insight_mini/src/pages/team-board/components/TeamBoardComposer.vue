<template>
  <view class="team-board-composer" @click.self="emit('close')">
    <view class="team-board-composer__dialog">
      <view class="section-heading section-heading--compact">
        <view>
          <text class="section-kicker">发布观点</text>
          <text class="section-title">{{ selectedPreset.label }}</text>
        </view>
        <button class="team-board-composer__close" @click="emit('close')">关闭</button>
      </view>

      <TeamBoardSnapshot :snapshot="selectedPreset.snapshot" composer />

      <input
        v-model="title"
        class="team-board-composer__input"
        maxlength="120"
        placeholder="例如：头部火力还是太集中"
      />

      <textarea
        v-model="commentary"
        class="team-board-composer__textarea"
        maxlength="280"
        placeholder="把你真正想说的判断写出来，不要复述数据。"
      />

      <text v-if="composerErrorMessage" class="team-board-composer__error">{{ composerErrorMessage }}</text>

      <button class="team-board-composer__submit" @click="emit('publish')">发布观点</button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TeamBoardComposerPreset } from '../../../types/teamBoard'
import TeamBoardSnapshot from './TeamBoardSnapshot.vue'

const props = defineProps<{
  selectedPreset: TeamBoardComposerPreset
  postTitle: string
  postCommentary: string
  composerErrorMessage: string
}>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'publish'): void
  (e: 'update:postTitle', value: string): void
  (e: 'update:postCommentary', value: string): void
}>()
const title = computed({
  get: () => props.postTitle,
  set: (value: string) => emit('update:postTitle', value),
})
const commentary = computed({
  get: () => props.postCommentary,
  set: (value: string) => emit('update:postCommentary', value),
})
</script>

<style scoped lang="css">
@import '../team-board-shared.css';

.team-board-composer__close {
  margin: 0 0 0 auto;
}
.team-board-composer {
  position: fixed;
  inset: 0;
  z-index: 30;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: var(--fi-space-28);
  background: rgba(21, 22, 27, 0.32);
  animation: fi-overlay-fade-in 180ms ease both;
}
.team-board-composer__dialog {
  width: 100%;
  max-height: 84vh;
  overflow-y: auto;
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}
.team-board-composer__textarea {
  min-height: 180rpx;
}
.team-board-composer__error {
  display: block;
  margin-top: var(--fi-space-12);
  color: #c03a2b;
  font-size: 24rpx;
}
</style>
