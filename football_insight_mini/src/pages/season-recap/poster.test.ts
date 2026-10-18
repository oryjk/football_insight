import { expect, test } from 'bun:test'
import { posterLines, saveRecapPoster } from './poster'
const summary={away_km:0,home_km:null,total_km:0,matches:0,home_matches:0,away_matches:0,cities:0}
test('poster keeps backend mileage and explains zero-away and unknown home distance',()=>{
 expect(posterLines(summary).includes('本赛季尚未远征')).toBe(true)
 expect(posterLines(summary).includes('未填写主场距离')).toBe(true)
 expect(posterLines({...summary,away_km:3101,home_km:123,total_km:3224}).includes('3101 公里')).toBe(true)
})
test('album permission denial offers settings and does not pretend saving succeeded',async()=>{
 let offered=false,opened=false,saved=false
 const platform={getSetting:(o:any)=>o.success({authSetting:{'scope.writePhotosAlbum':false}}),showModal:(o:any)=>{offered=true;o.success({confirm:true})},openSetting:()=>{opened=true},saveImageToPhotosAlbum:()=>{saved=true},showToast:()=>{}}
 await saveRecapPoster('poster.png',platform as any)
 expect(offered).toBe(true);expect(opened).toBe(true);expect(saved).toBe(false)
})
test('album saving failure is visible and success alone shows saved',async()=>{
 const messages:string[]=[]
 const platform={getSetting:(o:any)=>o.success({authSetting:{}}),showModal:()=>{},openSetting:()=>{},saveImageToPhotosAlbum:(o:any)=>o.fail({errMsg:'arbitrary failure'}),showToast:(o:any)=>messages.push(o.title)}
 await saveRecapPoster('poster.png',platform as any);expect(messages).toEqual(['保存失败，请重试'])
 platform.saveImageToPhotosAlbum=(o:any)=>o.success({});await saveRecapPoster('poster.png',platform as any);expect(messages.at(-1)).toBe('已保存到相册')
})
