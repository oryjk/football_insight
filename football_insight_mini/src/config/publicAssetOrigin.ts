const legacyAssetOrigin = 'https://oryjk.cn:82'
let publicAssetOrigin = legacyAssetOrigin
// #ifdef MP-WEIXIN
publicAssetOrigin = 'https://oryjk.cn:18443'
// #endif

export { publicAssetOrigin }

export function resolvePublicAssetUrl(url: string): string {
  const legacyBucket = `${legacyAssetOrigin}/minio/football-insight/`
  return url.startsWith(legacyBucket) ? `${publicAssetOrigin}${url.slice(legacyAssetOrigin.length)}` : url
}
