<template>
 <page-meta :page-style="navigationPageStyle" />
 <FiSecondaryNav @back="goBackFromSecondaryPage('/pages/user/index')" @share="copyPageShareLink(sharePath)" />
 <view class="recap-page">
  <FiPanel>
   <FiSectionHeading kicker="为蓉城走过的每一公里" :title="`${season} 我的赛季总结`" />
   <text class="caption">勾选去过的比赛，免费制作你的里程海报。记录会保存在账号中。</text>
  </FiPanel>
  <FiPanel v-if="!session.hasAccessToken.value">
   <text class="caption">登录后记录观赛足迹，换设备也能继续查看。</text>
   <button class="action" hover-class="pressed" @tap="login">登录并制作</button>
  </FiPanel>
  <FiLoading v-else-if="loading" title="正在整理赛季比赛" caption="只展示成都蓉城一线队已结束的比赛" />
  <FiPanel v-if="error"><text class="error">{{error}}</text><button v-if="!view && session.hasAccessToken.value" class="action" @tap="load">重新加载</button></FiPanel>
  <template v-if="view">
   <FiPanel>
    <FiSectionHeading kicker="第一步" :title="`勾选已去过的比赛 · ${selected.length} 场`" />
    <SeasonRecapMatches :matches="view.matches" :selected="selected" :disabled="generating" @toggle="toggle" />
   </FiPanel>
   <FiPanel>
    <FiSectionHeading kicker="第二步" title="去主场的单边距离" />
    <view class="distance-field"><input class="distance-input" type="digit" :value="distance" :disabled="generating" maxlength="6" placeholder="0–500" aria-label="去主场的单边距离，公里" @input="handleDistanceInput" /><text>公里</text></view>
    <text class="caption">填一次大概距离即可；没有选主场可以留空。最多两位小数。</text>
    <button class="action" :disabled="generating" hover-class="pressed" @tap="generate">{{generating?'正在制作海报…':'保存记录并生成海报'}}</button>
    <text class="caption">客场：成都至对手主场城市的大圆直线距离 × 2，每场取整后相加。主场：单边距离 × 2 × 场次，合计取整。取消或未结束的比赛不计入。</text>
   </FiPanel>
   <FiPanel v-if="posterPath">
    <FiSectionHeading kicker="我的赛季足迹" title="这一年，与你同场" />
    <image :src="posterPath" class="poster" mode="widthFix" @tap="preview" />
    <!-- #ifdef MP-WEIXIN -->
    <button class="action" hover-class="pressed" @tap="saveImage">保存海报到相册</button>
    <button class="action action--secondary" open-type="share" hover-class="pressed">分享我的赛季总结</button>
    <!-- #endif -->
    <!-- #ifndef MP-WEIXIN -->
    <button class="action" hover-class="pressed" @tap="preview">查看大图，长按保存</button>
    <button class="action action--secondary" hover-class="pressed" @tap="copyPageShareLink(sharePath)">复制分享链接</button>
    <!-- #endif -->
   </FiPanel>
  </template>
  <SeasonRecapPoster v-if="generated" :key="generated.key" :payload="generated" @ready="posterReady" @failed="posterFailed" />
 </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { onLoad,onShow,onUnload,onShareAppMessage,onShareTimeline } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiPanel from '../../components/FiPanel.vue'
import FiSectionHeading from '../../components/FiSectionHeading.vue'
import FiLoading from '../../components/FiLoading.vue'
import SeasonRecapMatches from './components/SeasonRecapMatches.vue'
import SeasonRecapPoster from './components/SeasonRecapPoster.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { getSeasonRecap,saveSeasonRecap,getSeasonRecapCode } from '../../api/seasonRecap'
import { setPostLoginRedirectTarget } from '../../utils/authStorage'
import { copyPageShareLink,goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { recapSharePath,resolveRecapSeason } from './helpers'
import { createRecapFlow } from './flow'
import { saveRecapPoster } from './poster'
const session=useAppSession()
const {navigationPageStyle}=useNavigationLayout()
const flow=createRecapFlow(session,{load:getSeasonRecap,save:saveSeasonRecap,code:getSeasonRecapCode})
const {season,view,selected,distance,loading,generating,error,posterPath,generated,load,toggle,editDistance,generate,posterReady,posterFailed}=flow
const sharePath=computed(()=>recapSharePath(season.value))
onLoad(query=>{season.value=resolveRecapSeason(query??{})})
onShow(()=>{if(!view.value)void load()})
onUnload(flow.dispose)
// vue-tsc resolves <input> to the H5 InputEvent; uni-app supplies the same detail.value payload on both runtimes.
function handleDistanceInput(event: InputEvent) {
 const detail = event.detail as unknown as { value: string }
 editDistance(detail.value)
}
function login(){setPostLoginRedirectTarget({url:sharePath.value,type:'navigateTo'});uni.switchTab({url:'/pages/user/index'})}
function preview(){if(posterPath.value)uni.previewImage({urls:[posterPath.value]})}
function saveImage(){void saveRecapPoster(posterPath.value)}
// Share metadata contains no account or selection identifiers; recipients make their own recap.
onShareAppMessage(()=>({title:`我的 ${season.value} 蓉城赛季足迹`,path:sharePath.value,...(posterPath.value?{imageUrl:posterPath.value}:{})}))
onShareTimeline(()=>({title:`我的 ${season.value} 蓉城赛季足迹`,query:`season=${season.value}&src=share&cid=recap${season.value}`,...(posterPath.value?{imageUrl:posterPath.value}:{})}))
</script>
<style scoped>
.recap-page {min-height:100vh;box-sizing:border-box;display:flex;flex-direction:column;gap:var(--fi-space-card-gap);padding:calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-24) calc(var(--fi-space-40) + env(safe-area-inset-bottom));background:var(--fi-color-page);color:var(--fi-color-text-primary);}
.caption {display:block;margin-top:var(--fi-space-16);color:var(--fi-color-text-secondary);font-size:var(--fi-font-24);line-height:var(--fi-leading-normal);}
.error {color:var(--fi-color-primary);font-size:var(--fi-font-28);line-height:var(--fi-leading-normal);}
.action {margin:var(--fi-space-20) 0 0;padding:var(--fi-space-20) var(--fi-space-24);background:var(--fi-color-primary);color:var(--fi-color-surface);border:0;border-radius:var(--fi-radius-round);font-size:var(--fi-font-28);font-weight:var(--fi-weight-bold);line-height:var(--fi-leading-normal);}
.action::after{border:0;}.action--secondary {background:var(--fi-color-page);color:var(--fi-color-text-primary);}.pressed{opacity:var(--fi-opacity-pressed);}
.action[disabled]{background:var(--fi-color-divider);color:var(--fi-color-text-muted);}
.distance-field {display:flex;align-items:center;gap:var(--fi-space-16);margin-top:var(--fi-space-20);font-size:var(--fi-font-28);}
.distance-input {flex:1;height:var(--fi-space-72);box-sizing:border-box;padding:0 var(--fi-space-20);background:var(--fi-color-page);border-radius:var(--fi-radius-sm);font-size:var(--fi-font-32);line-height:var(--fi-leading-normal);}
.poster {width:100%;display:block;margin-top:var(--fi-space-20);border-radius:var(--fi-radius-md);}
</style>
