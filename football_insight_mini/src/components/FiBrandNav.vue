<template>
    <view class="fi-brand-nav" :class="{ 'fi-brand-nav--transparent': transparent, 'fi-brand-nav--h5': isH5 }">
        <view class="fi-brand-nav__status-spacer"></view>
        <view class="fi-brand-nav__bar">
            <view class="fi-brand-nav__brand" :style="brandStyle">
                <button v-if="showBack" class="fi-brand-nav__back" aria-label="返回" @tap="emit('back')">
                    <text aria-hidden="true">‹</text>
                </button>
                <image
                    :src="logo"
                    mode="aspectFill"
                    class="fi-brand-nav__logo"
                />
                <view class="fi-brand-nav__copy">
                    <text class="fi-brand-nav__name">足球洞察</text>
                    <text class="fi-brand-nav__tagline">{{ subtitle }}</text>
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
        showBack?: boolean;
        subtitle?: string;
    }>(),
    {
        transparent: false,
        showBack: false,
        subtitle: "最懂球迷的工具",
    },
);

const emit = defineEmits<{ (e: "back"): void }>();

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
    background: var(--fi-color-brand-nav-bg);
    border-bottom: 1rpx solid var(--fi-color-brand-nav-border);
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
    color: var(--fi-color-bg);
    text-shadow: var(--fi-shadow-brand-nav-text);
}

.fi-brand-nav--transparent .fi-brand-nav__tagline {
    color: var(--fi-color-brand-nav-inverse);
    text-shadow: var(--fi-shadow-brand-nav-tagline);
}

.fi-brand-nav--transparent .fi-brand-nav__logo {
    border-color: var(--fi-color-brand-nav-inverse);
    box-shadow: var(--fi-shadow-brand-nav-logo-inverse);
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
    border: 2rpx solid var(--fi-color-brand-nav-logo-border);
    box-shadow: var(--fi-shadow-brand-nav-logo);
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
    color: var(--fi-color-text-primary);
    font-size: 32rpx;
    font-weight: 900;
    line-height: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.fi-brand-nav__tagline {
    color: var(--fi-color-text-muted);
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
.fi-brand-nav__back {
    flex-shrink: 0;
    width: var(--fi-space-44);
    height: var(--fi-space-brand-nav-logo-size);
    display: flex;
    align-items: center;
    justify-content: flex-start;
    margin: 0;
    padding: 0;
    background: transparent;
    color: var(--fi-color-text-primary);
    font-size: var(--fi-font-48);
    line-height: var(--fi-leading-none);
}
.fi-brand-nav__back::after { border: 0; }
.fi-brand-nav--transparent .fi-brand-nav__back { color: var(--fi-color-bg); }
</style>
