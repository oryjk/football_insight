import type { RecapSummary, RecapCodeImage } from '../../types/seasonRecap'
export const POSTER_WIDTH = 750
export const POSTER_HEIGHT = 1080
// Decorative exported bitmap palette, independent of live UI tokens (design-system canvas exception).
export const POSTER_PALETTE = { paper: '#ffffff', ink: '#15161b', muted: '#6d7280', accent: '#dc2626', background: '#f6f7fb' }
export function posterLines(summary: RecapSummary): string[] {
 return [`${summary.away_km} 公里`,summary.away_matches===0?'本赛季尚未远征':'客场远征 · 城市间大圆直线往返',summary.home_km===null?'未填写主场距离':`${summary.home_km} 公里`,`${summary.matches} 场比赛 · ${summary.cities} 座城市`]
}
type AlbumPlatform = Pick<typeof uni, 'getSetting' | 'showModal' | 'openSetting' | 'saveImageToPhotosAlbum' | 'showToast'>
export async function saveRecapPoster(path: string, platform: AlbumPlatform = uni): Promise<void> {
 if(!path)return
 const denied=()=>new Promise<boolean>(resolve=>platform.getSetting({success:result=>resolve(result.authSetting['scope.writePhotosAlbum']===false),fail:()=>resolve(false)}))
 const offerSettings=()=>platform.showModal({title:'需要相册权限',content:'请在设置中允许保存到相册，返回后再次点击保存。',confirmText:'去设置',success:result=>{if(result.confirm)platform.openSetting({})}})
 if(await denied()){offerSettings();return}
 await new Promise<void>(resolve=>platform.saveImageToPhotosAlbum({filePath:path,success:()=>{platform.showToast({title:'已保存到相册',icon:'success'});resolve()},fail:async()=>{if(await denied())offerSettings();else platform.showToast({title:'保存失败，请重试',icon:'none'});resolve()}}))
}
/** Canvas image input: a temporary local file on WeChat; a data URL on H5. */
export async function materializeCode(code: RecapCodeImage): Promise<{path:string;cleanup:()=>void}> {
 // #ifdef MP-WEIXIN
 const fs=uni.getFileSystemManager()
 const extension=code.mime_type==='image/jpeg'?'jpg':'png'
 const path=`${(uni as typeof uni & {env:{USER_DATA_PATH:string}}).env.USER_DATA_PATH}/fi-recap-code-${Date.now()}.${extension}`
 await new Promise<void>((resolve,reject)=>fs.writeFile({filePath:path,data:code.image_base64,encoding:'base64',success:()=>resolve(),fail:reject}))
 return {path,cleanup:()=>{fs.unlink({filePath:path,fail:()=>{}})}}
 // #endif
 // #ifndef MP-WEIXIN
 return {path:`data:${code.mime_type};base64,${code.image_base64}`,cleanup:()=>{}}
 // #endif
}
