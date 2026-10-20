<template>
 <canvas canvas-id="seasonRecapCanvas" class="recap-canvas" :style="canvasStyle" />
</template>
<script setup lang="ts">
import { getCurrentInstance, onMounted, nextTick } from 'vue'
import { exportCanvasToTempFile, flushCanvas } from '../../../utils/canvasPoster'
import type { RecapPosterPayload } from '../flow'
import { buildRecapPosterContent, materializeCode, POSTER_WIDTH, POSTER_HEIGHT } from '../poster'
import { drawRecapPoster } from '../posterDrawing'
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
  const code=await materializeCode(payload.code);cleanup=code.cleanup
  const image=await new Promise<UniApp.GetImageInfoSuccessData>((resolve,reject)=>uni.getImageInfo({src:code.path,success:resolve,fail:reject}))
  const context=uni.createCanvasContext('seasonRecapCanvas',instance)
  drawRecapPoster(context,buildRecapPosterContent(payload.view),image.path)
  await flushCanvas(context)
  const path=await exportCanvasToTempFile({canvasId:'seasonRecapCanvas',width:POSTER_WIDTH,height:POSTER_HEIGHT,instance})
  emit('ready',payload.key,path)
 } catch {emit('failed',payload.key)} finally {cleanup()}
})
</script>
<style scoped>
.recap-canvas { position: fixed; left: -10000rpx; top: -10000rpx; pointer-events: none; }
</style>
