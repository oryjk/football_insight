<template>
  <div v-if="visible" class="confirm">
    <div class="confirm__scrim" @click="emit('cancel')"></div>
    <div class="confirm__panel">
      <span class="confirm__title">{{ title }}</span>
      <span class="confirm__content">{{ content }}</span>
      <div class="confirm__footer">
        <button type="button" class="confirm__btn confirm__btn--ghost" @click="emit('cancel')">再想想</button>
        <button type="button" class="confirm__btn confirm__btn--primary" @click="emit('confirm')">{{ confirmText }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean
  title: string
  content: string
  confirmText: string
}>()

const emit = defineEmits<{
  (e: 'confirm'): void
  (e: 'cancel'): void
}>()
</script>

<style scoped>
.confirm {
  position: fixed;
  inset: 0;
  z-index: 120;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.confirm__scrim {
  position: absolute;
  inset: 0;
  background: rgba(15, 17, 22, 0.46);
}

.confirm__panel {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 8px;
  width: 100%;
  max-width: 340px;
  padding: 20px;
  border-radius: 12px;
  background: #fff;
  box-shadow: 0 16px 36px rgba(12, 14, 20, 0.24);
}

.confirm__title {
  color: #17181c;
  font-size: 16px;
  font-weight: 800;
}

.confirm__content {
  color: #5b5f6a;
  font-size: 13px;
  line-height: 1.6;
}

.confirm__footer {
  display: flex;
  gap: 10px;
  margin-top: 8px;
}

.confirm__btn {
  flex: 1;
  padding: 8px 0;
  border-radius: 999px;
  border: 0;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.confirm__btn--primary {
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
}

.confirm__btn--ghost {
  background: #fff;
  border: 1px solid rgba(32, 36, 44, 0.16);
  color: #5b5f6a;
}
</style>
