import { ref, watch, computed, type Ref } from 'vue'
import type { SeasonRecap, SaveSeasonRecap, RecapCodeImage } from '../../types/seasonRecap'
import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
import { parseHomeDistance } from './helpers'
import type { RecapPosterTheme } from './poster'
interface Client {
 load(season: number): Promise<SeasonRecap>
 save(season: number, payload: SaveSeasonRecap): Promise<SeasonRecap>
 code(season: number): Promise<RecapCodeImage>
}
export interface RecapPosterPayload { key: number; view: SeasonRecap; code: RecapCodeImage; theme: RecapPosterTheme }
export function createRecapFlow(session: { revision: Ref<number>; hasAccessToken: Ref<boolean>; invalidateSession?: (expectedRevision: number) => void }, client: Client) {
 const theme = ref<RecapPosterTheme>('light')
 const season = ref(2026), view = ref<SeasonRecap | null>(null), selected = ref<number[]>([]), distance = ref('')
 const loading = ref(false), generating = ref(false), error = ref(''), posterPath = ref('')
 const generated = ref<RecapPosterPayload | null>(null), key = ref(0)
 let operation = 0
 function invalidate() { operation++; key.value++; view.value=null;selected.value=[];distance.value='';generated.value=null;posterPath.value='';loading.value=false;generating.value=false;error.value='' }
 const stop=watch([session.revision,session.hasAccessToken],()=>{
  invalidate()
  const pending=operation
  // Auth revision may fire before the credential store finishes changing; read it next microtask.
  void Promise.resolve().then(()=>{if(pending===operation && session.hasAccessToken.value)void load()})
 },{flush:'sync'})
 function message(cause: unknown) { return cause instanceof ApiRequestError && cause.statusCode===401 ? '请先登录后制作赛季总结' : extractApiErrorMessage(cause,'赛季总结暂不可用，请稍后重试') }
 function handleError(cause: unknown, revision: number) {
  const text=message(cause)
  if(cause instanceof ApiRequestError && cause.statusCode===401 && revision===session.revision.value) session.invalidateSession?.(revision)
  error.value=text
 }
 async function load() {
  if (!session.hasAccessToken.value) { invalidate();error.value='请先登录后制作赛季总结';return }
  const id=++operation, revision=session.revision.value
  loading.value=true;error.value=''
  try { const result=await client.load(season.value);if(id!==operation||revision!==session.revision.value)return
   view.value=result;selected.value=[...result.selected_match_ids];distance.value=result.home_one_way_km===null?'':String(result.home_one_way_km)
  } catch(cause) {if(id===operation) handleError(cause,revision)} finally {if(id===operation) loading.value=false}
 }
 function invalidatePoster() { key.value++; generated.value=null;posterPath.value='' }
 function toggle(id: number) {
  if (generating.value || !view.value?.matches.some(m=>m.match_id===id)) return
  selected.value=selected.value.includes(id)?selected.value.filter(item=>item!==id):[...selected.value,id];invalidatePoster()
 }
 function setSelected(ids: number[]) {
  if (generating.value || !view.value) return
  const known = new Set(view.value.matches.map(m=>m.match_id))
  selected.value=[...new Set(ids.filter(id=>known.has(id)))];invalidatePoster()
 }
 const dirty = computed(() => {
  const current = view.value
  if (!current) return false
  const sort = (ids: number[]) => [...new Set(ids)].sort((a,b)=>a-b)
  const picked = sort(selected.value), saved = sort(current.selected_match_ids)
  if (picked.length !== saved.length || picked.some((id,index)=>id!==saved[index])) return true
  const km = distance.value.trim() === '' ? null : Number(distance.value)
  return km !== current.home_one_way_km
 })
 function editDistance(value: string) { if(generating.value)return;distance.value=value;invalidatePoster() }
 function setTheme(value: RecapPosterTheme) {
  if (generating.value || theme.value === value) return
  theme.value = value; invalidatePoster()
 }
 async function generate() {
  if(generating.value||loading.value||!view.value||!session.hasAccessToken.value)return
  error.value='';invalidatePoster()
  let km: number | null
  try {km=parseHomeDistance(distance.value,view.value.matches.some(m=>m.is_home&&selected.value.includes(m.match_id)))} catch(cause) {error.value=message(cause);return}
  const id=++operation, revision=session.revision.value
  generating.value=true
  try {
   const result=await client.save(season.value,{selected_match_ids:[...new Set(selected.value)],home_one_way_km:km})
   if(id!==operation||revision!==session.revision.value)return
   view.value=result
   const code=await client.code(season.value)
   if(id!==operation||revision!==session.revision.value)return
   generated.value={key:key.value,view:result,code,theme:theme.value}
  } catch(cause) {if(id===operation){handleError(cause,revision);generating.value=false}}
 }
 function posterReady(generation: number, path: string) {if(generation!==key.value||!generated.value)return;posterPath.value=path;generating.value=false}
 function posterFailed(generation: number) {if(generation!==key.value)return;error.value='海报生成失败，请重试';generated.value=null;generating.value=false}
 function dispose() {stop();invalidate()}
 return {theme,setTheme,season,view,selected,distance,loading,generating,error,posterPath,generated,key,client,dirty,load,toggle,setSelected,editDistance,generate,posterReady,posterFailed,dispose}
}
