<template>
 <canvas canvas-id="seasonRecapCanvas" class="recap-canvas" :style="canvasStyle" />
</template>
<script setup lang="ts">
import { getCurrentInstance, onMounted, nextTick } from 'vue'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL, SEASON_RECAP_POSTER_BACKGROUNDS } from '../../../config/assets'
import { exportCanvasToTempFile, flushCanvas } from '../../../utils/canvasPoster'
import type { RecapPosterPayload } from '../flow'
import { buildRecapPosterContent, materializeCode, POSTER_WIDTH, POSTER_HEIGHT, supportsRecapWebp } from '../poster'
import { drawRecapPoster } from '../posterDrawing'
import { loadRecapPosterImages } from '../posterImages'
const props=defineProps<{payload:RecapPosterPayload}>()
const emit=defineEmits<{(event:'ready',key:number,path:string):void;(event:'failed',key:number):void}>()
const instance=getCurrentInstance()
// Export bitmap geometry is in pixels; hidden canvas must match drawing coordinates on both runtimes.
const canvasStyle={width:`${POSTER_WIDTH}px`,height:`${POSTER_HEIGHT}px`}
onMounted(async()=>{
 const payload=props.payload
 let cleanup=()=>{}
 try {
  await nextTick()
  const content=buildRecapPosterContent(payload.view)
  let allowWebp=false
  try { const info=uni.getSystemInfoSync();allowWebp=supportsRecapWebp(info.platform,info.system) } catch { /* use PNG/JPEG only */ }
  const decorations=loadRecapPosterImages([
   SEASON_RECAP_POSTER_BACKGROUNDS[payload.theme ?? 'dark'],
   CHENGDU_RONGCHENG_CREST_IMAGE_URL,
   ...(content.routeEntries?.entries.flatMap(entry=>entry.avatarUrls) ?? []),
  ],allowWebp)
  const code=await materializeCode(payload.code);cleanup=code.cleanup
  const image=await new Promise<UniApp.GetImageInfoSuccessData>((resolve,reject)=>uni.getImageInfo({src:code.path,success:resolve,fail:reject}))
  const context=uni.createCanvasContext('seasonRecapCanvas',instance)
  drawRecapPoster(context,content,image.path,await decorations,payload.theme)
  await flushCanvas(context)
  const path=await exportCanvasToTempFile({canvasId:'seasonRecapCanvas',width:POSTER_WIDTH,height:POSTER_HEIGHT,instance})
  emit('ready',payload.key,path)
 } catch {emit('failed',payload.key)} finally {cleanup()}
})
</script>
<style scoped>
.recap-canvas { position: fixed; left: -10000rpx; top: -10000rpx; pointer-events: none; }
</style>
