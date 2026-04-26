import { WECHAT_CLOUD_ENV_ID } from '../config/aiChat'

type WechatCloud = {
  init?: (options: { env: string }) => void
  extend?: {
    AI?: unknown
  }
}

let hasInitializedWechatCloud = false

function getWechatCloud(): WechatCloud | null {
  const wxLike = (globalThis as { wx?: { cloud?: WechatCloud } }).wx
  return wxLike?.cloud ?? null
}

export function initWechatCloud(): void {
  const cloud = getWechatCloud()
  if (!cloud?.init || hasInitializedWechatCloud) {
    return
  }

  cloud.init({
    env: WECHAT_CLOUD_ENV_ID,
  })
  hasInitializedWechatCloud = true
}

export function getWechatCloudAiExtension(): any | null {
  return getWechatCloud()?.extend?.AI ?? null
}

export async function callWechatCloudFunction<TResponse>(
  name: string,
  data: Record<string, unknown>,
): Promise<TResponse> {
  const cloud = getWechatCloud()
  if (!cloud || typeof (cloud as any).callFunction !== 'function') {
    throw new Error('当前小程序环境不支持微信云函数')
  }

  const result = await (cloud as any).callFunction({
    name,
    data,
  })

  return (result?.result ?? null) as TResponse
}
