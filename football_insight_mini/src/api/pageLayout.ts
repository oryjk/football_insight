import type { PageLayoutKey, PublicPageLayout, PublicPageLayoutsResponse } from '../types/pageLayout'
import { request } from '../utils/request'

export function getPageLayouts(): Promise<PublicPageLayoutsResponse> {
  return request<PublicPageLayoutsResponse>({ url: '/page-layouts' })
}

export function getPageLayout(pageKey: PageLayoutKey): Promise<PublicPageLayout> {
  return request<PublicPageLayout>({ url: `/page-layouts/${encodeURIComponent(pageKey)}` })
}
