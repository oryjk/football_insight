export function request<T>(options: { url: string }): Promise<T> {
  return new Promise((resolve) => {
    uni.request({ ...options, success: () => resolve(null as T) })
  })
}
