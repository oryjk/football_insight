import { expect, test } from 'bun:test'
import { loadRecapPosterImages } from './posterImages'

test('decorations load concurrently, dedupe URLs and skip failed or invalid images', async () => {
 const requested: string[] = []
 const callbacks: UniApp.GetImageInfoOptions[] = []
 const platform = { getImageInfo(options: UniApp.GetImageInfoOptions) { requested.push(options.src); callbacks.push(options) } }
 const pending = loadRecapPosterImages(['bg.jpg','crest.png','bg.jpg','bad.png','zero.png'], true, platform)
 expect(requested).toEqual(['bg.jpg','crest.png','bad.png','zero.png'])
 callbacks[1]!.success!({path:'/local/crest',width:80,height:80,type:'png'} as UniApp.GetImageInfoSuccessData)
 callbacks[0]!.success!({path:'/local/bg',width:750,height:1334,type:'jpeg'} as UniApp.GetImageInfoSuccessData)
 callbacks[2]!.fail!({errMsg:'download failed'})
 callbacks[3]!.success!({path:'/local/zero',width:0,height:80} as UniApp.GetImageInfoSuccessData)
 expect([...(await pending)]).toEqual([
  ['bg.jpg',{path:'/local/bg',width:750,height:1334}],
  ['crest.png',{path:'/local/crest',width:80,height:80}],
 ])
})

test('missing callbacks time out and late callbacks cannot change the completed map', async () => {
 let callback: UniApp.GetImageInfoOptions | undefined
 const pending = loadRecapPosterImages(['timeout.png'],true,{getImageInfo(options: UniApp.GetImageInfoOptions) { callback=options }},10)
 const images = await pending
 expect(images.size).toBe(0)
 callback!.success!({path:'/late',width:10,height:10} as UniApp.GetImageInfoSuccessData)
 expect(images.size).toBe(0)
})

test('synchronous errors and unsupported WebP degrade without rejecting', async () => {
 const requested: string[]=[]
 const images=await loadRecapPosterImages(['a.webp?v=1','disguised.png','throw.png'],false,{getImageInfo(options: UniApp.GetImageInfoOptions) {
  requested.push(options.src)
  if(options.src==='throw.png') throw new Error('unsupported')
  options.success!({path:'/webp',width:10,height:10,type:'webp'} as UniApp.GetImageInfoSuccessData)
 }})
 expect(requested).toEqual(['disguised.png','throw.png'])
 expect(images.size).toBe(0)
})
