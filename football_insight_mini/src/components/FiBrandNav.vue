<template>
    <view class="fi-brand-nav" :class="{ 'fi-brand-nav--transparent': transparent, 'fi-brand-nav--h5': isH5 }">
        <view class="fi-brand-nav__status-spacer"></view>
        <view class="fi-brand-nav__bar">
            <view class="fi-brand-nav__brand" :style="brandStyle">
                <image
                    :src="logo"
                    mode="aspectFill"
                    class="fi-brand-nav__logo"
                />
                <view class="fi-brand-nav__copy">
                    <text class="fi-brand-nav__name">足球洞察</text>
                    <text class="fi-brand-nav__tagline">最懂球迷的工具</text>
                </view>
            </view>
        </view>
    </view>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import logo from "../static/app-logo.jpg";

withDefaults(
    defineProps<{
        transparent?: boolean;
    }>(),
    {
        transparent: false,
    },
);

const isH5 =
    // #ifdef H5
    true
    // #endif
    // #ifndef H5
    false
    // #endif
;

// 胶囊指标仅在小程序端可用；初始为 null 时不输出内联样式，交给 CSS 兜底，
// 避免 H5 上错误地沿用 px 数值导致品牌错位。
const brandMetrics = ref<{
    top: number;
    height: number;
} | null>(null);

const brandStyle = computed(() => {
    const metrics = brandMetrics.value;
    if (!metrics) {
        return {};
    }

    return {
        top: metrics.top ? `${metrics.top}px` : "",
        height: `${metrics.height}px`,
    };
});

// 品牌与微信菜单胶囊垂直居中对齐；H5 没有胶囊，走 CSS 流式布局，不需要同步位置。
function syncBrandWithMenuCapsule(): void {
    // #ifdef H5
    return;
    // #endif

    try {
        const menuButton = uni.getMenuButtonBoundingClientRect?.();

        if (!menuButton?.width || !menuButton?.height) {
            return;
        }

        const brandHeight = 64;
        brandMetrics.value = {
            top: menuButton.top + (menuButton.height - brandHeight) / 2,
            height: brandHeight,
        };
    } catch {
        // Keep the CSS fallback when the platform does not expose capsule metrics.
    }
}

onMounted(() => {
    syncBrandWithMenuCapsule();
});
</script>

<style scoped>
.fi-brand-nav {
    --fi-brand-nav-status-height: var(--status-bar-height);
    --fi-brand-nav-bar-height: 154rpx;
    position: fixed;
    left: 0;
    right: 0;
    top: 0;
    z-index: 80;
    background: rgba(247, 248, 250, 0.82);
    border-bottom: 1rpx solid rgba(232, 233, 238, 0.72);
    backdrop-filter: blur(18rpx);
    -webkit-backdrop-filter: blur(18rpx);
}

.fi-brand-nav--transparent {
    background: transparent;
    border-bottom-color: transparent;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
}

.fi-brand-nav--transparent .fi-brand-nav__name {
    color: #ffffff;
    text-shadow: 0 4rpx 12rpx rgba(0, 0, 0, 0.36);
}

.fi-brand-nav--transparent .fi-brand-nav__tagline {
    color: rgba(255, 255, 255, 0.82);
    text-shadow: 0 4rpx 12rpx rgba(0, 0, 0, 0.34);
}

.fi-brand-nav--transparent .fi-brand-nav__logo {
    border-color: rgba(255, 255, 255, 0.82);
    box-shadow: 0 10rpx 24rpx rgba(0, 0, 0, 0.22);
}

.fi-brand-nav__status-spacer {
    height: var(--fi-brand-nav-status-height);
}

.fi-brand-nav__bar {
    position: relative;
    min-height: var(--fi-brand-nav-bar-height);
    padding: 14rpx 196rpx 48rpx 24rpx;
}

.fi-brand-nav__brand {
    position: fixed;
    left: 24rpx;
    top: calc(var(--fi-brand-nav-status-height) + 14rpx);
    z-index: 81;
    min-width: 0;
    width: 360rpx;
    height: 64rpx;
    display: flex;
    align-items: center;
    gap: 14rpx;
}

.fi-brand-nav__logo {
    flex: 0 0 auto;
    width: 64rpx;
    height: 64rpx;
    border-radius: 18rpx;
    border: 2rpx solid rgba(255, 255, 255, 0.78);
    box-shadow: 0 10rpx 22rpx rgba(21, 22, 27, 0.1);
}

.fi-brand-nav__copy {
    min-width: 0;
    height: 64rpx;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 5rpx;
}

.fi-brand-nav__name {
    color: #15161b;
    font-size: 32rpx;
    font-weight: 900;
    line-height: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.fi-brand-nav__tagline {
    color: #8f9198;
    font-size: 20rpx;
    font-weight: 500;
    line-height: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* H5 没有右上角胶囊，品牌回到正常流式布局。 */
.fi-brand-nav--h5 .fi-brand-nav__bar {
    min-height: auto;
    padding: 20rpx 24rpx;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24rpx;
}

.fi-brand-nav--h5 .fi-brand-nav__brand {
    position: static;
    width: auto;
    max-width: 60%;
}
</style>
