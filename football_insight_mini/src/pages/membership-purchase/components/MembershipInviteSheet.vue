<template>
  <FiBottomSheet
    :visible="visible"
    title="邀请好友免费升级"
    height="82vh"
    @update:visible="emit('update:visible', $event)"
  >
    <view v-if="inviteCode" class="invite-code">
      <view class="invite-code__body">
        <text class="invite-code__label">我的邀请码</text>
        <text class="invite-code__value">{{ inviteCode }}</text>
      </view>
      <button class="invite-code__copy" hover-class="invite-code__copy--pressed" @tap="emit('copy')">复制</button>
    </view>
    <view v-else class="invite-code invite-code--empty">
      <text class="invite-code__empty">你的账号注册时没有填邀请码，暂时没有可以发给好友的邀请码，邀请升级暂不可用。</text>
    </view>

    <text class="invite-section">怎么邀请</text>
    <view class="invite-steps">
      <view class="invite-step">
        <text class="invite-step__no">1</text>
        <text class="invite-step__text">把你的邀请码发给好友，请好友搜索打开小程序「洞察足球集散地」。</text>
      </view>
      <view class="invite-step">
        <text class="invite-step__no">2</text>
        <view class="invite-step__body">
          <text class="invite-step__text">好友第一次登录时会看到两个输入框：</text>
          <text class="invite-step__text"><text class="invite-step__strong">「推荐码」</text>填你的邀请码，这一步才算你邀请成功；</text>
          <text class="invite-step__text"><text class="invite-step__strong">「邀请码」</text>填好友自己的邀请码，可不填。好友关注公众号「何止编程」回复“邀请码”就能领取，填了好友自己直接是 V3。</text>
          <view class="invite-step__action" @tap="emit('open-official')">
            <text>打开公众号「何止编程」</text>
            <text class="invite-step__arrow">›</text>
          </view>
        </view>
      </view>
      <view class="invite-step">
        <text class="invite-step__no">3</text>
        <text class="invite-step__text">好友登录成功，你的邀请人数加 1；人数够了自动升级，不用申请。</text>
      </view>
    </view>

    <template v-if="milestones.length">
      <text class="invite-section">邀请多少人升到几级</text>
      <view class="invite-table">
        <view class="invite-table__head">
          <text class="invite-table__col">邀请人数</text>
          <text class="invite-table__col">升到</text>
          <text class="invite-table__col invite-table__col--end">刷新</text>
        </view>
        <view v-for="item in milestones" :key="item.code" class="invite-table__row">
          <text class="invite-table__col invite-table__strong">{{ item.referrals }} 人</text>
          <text class="invite-table__col">{{ item.code }}</text>
          <text class="invite-table__col invite-table__col--end">{{ item.refreshLabel }}</text>
        </view>
      </view>
    </template>

    <text class="invite-section">注意</text>
    <view class="invite-notes">
      <text class="invite-note">· 只有好友第一次登录时填写的推荐码才算，登录后不能补填；每位好友只算一次。</text>
      <text class="invite-note">· 升级后有效期一年，从升级当天算起；到期后回到 V3。</text>
      <text class="invite-note">· 你已经是更高等级（比如已开通会员）时，邀请人数不会让等级降低。</text>
      <text class="invite-note">· 邀请升级的会员权益，需要绑定并关注公众号「何止编程」后才生效。</text>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type { InviteMilestone } from '../../../utils/membershipRules'

defineProps<{
  visible: boolean
  inviteCode: string
  milestones: InviteMilestone[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'copy'): void
  (e: 'open-official'): void
}>()
</script>

<style scoped lang="css">
.invite-code {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  padding: var(--fi-space-20) var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  background: rgba(var(--fi-primitive-red-rgb), 0.05);
}

.invite-code--empty {
  background: var(--fi-color-page);
}

.invite-code__body {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
  min-width: 0;
}

.invite-code__label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.invite-code__value {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
  letter-spacing: 2rpx;
}

.invite-code__copy {
  flex-shrink: 0;
  margin: 0;
  height: 60rpx;
  line-height: 60rpx;
  padding: 0 var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.invite-code__copy--pressed {
  opacity: 0.8;
}

.invite-code__empty {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.invite-section {
  display: block;
  margin-top: var(--fi-space-28);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.invite-steps {
  margin-top: var(--fi-space-14);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
}

.invite-step {
  display: flex;
  align-items: flex-start;
  gap: var(--fi-space-14);
}

.invite-step__no {
  flex-shrink: 0;
  width: 36rpx;
  height: 36rpx;
  line-height: 36rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  text-align: center;
}

.invite-step__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.invite-step__text {
  flex: 1;
  min-width: 0;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.invite-step__strong {
  color: var(--fi-color-text-strong);
  font-weight: var(--fi-weight-bold);
}

.invite-step__action {
  align-self: flex-start;
  display: flex;
  align-items: center;
  gap: var(--fi-space-6);
  margin-top: var(--fi-space-4);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-medium);
}

.invite-step__arrow {
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-none);
}

.invite-table {
  margin-top: var(--fi-space-14);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-divider);
  overflow: hidden;
}

.invite-table__head,
.invite-table__row {
  display: flex;
  align-items: center;
  padding: var(--fi-space-16) var(--fi-space-20);
}

.invite-table__head {
  background: var(--fi-color-page);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.invite-table__row {
  border-top: var(--fi-border-divider);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.invite-table__col {
  flex: 1;
}

.invite-table__col--end {
  text-align: right;
}

.invite-table__strong {
  color: var(--fi-color-text-strong);
  font-weight: var(--fi-weight-bold);
}

.invite-notes {
  margin-top: var(--fi-space-14);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-10);
}

.invite-note {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}
</style>
