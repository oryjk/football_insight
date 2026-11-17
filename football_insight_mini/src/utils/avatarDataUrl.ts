export function inferAvatarMimeType(filePath: string): string {
  const lowerPath = filePath.toLowerCase()
  if (lowerPath.endsWith('.jpg') || lowerPath.endsWith('.jpeg')) {
    return 'image/jpeg'
  }
  if (lowerPath.endsWith('.webp')) {
    return 'image/webp'
  }
  return 'image/png'
}

function compressOnMini(filePath: string, quality: number): Promise<string> {
  return new Promise((resolve) => {
    if (typeof uni === 'undefined' || typeof uni.compressImage !== 'function') {
      resolve(filePath)
      return
    }
    uni.compressImage({
      src: filePath,
      quality,
      compressedWidth: 256,
      success: (res) => resolve(res.tempFilePath || filePath),
      fail: () => resolve(filePath),
    })
  })
}

function readMiniFileBase64(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    uni.getFileSystemManager().readFile({
      filePath,
      encoding: 'base64',
      success: (result) => {
        const base64 = typeof result.data === 'string' ? result.data : ''
        if (!base64) {
          reject(new Error('头像读取失败'))
          return
        }
        resolve(`data:${inferAvatarMimeType(filePath)};base64,${base64}`)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '头像读取失败'))
      },
    })
  })
}

// #ifdef H5
async function readH5BlobAsDataUrl(filePath: string): Promise<string> {
  let blob: Blob
  try {
    const response = await fetch(filePath)
    if (!response.ok) {
      throw new Error('头像读取失败，请重新选择')
    }
    blob = await response.blob()
  } catch {
    throw new Error('头像读取失败，请重新选择')
  }

  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      if (typeof reader.result === 'string' && reader.result.startsWith('data:image/')) {
        resolve(reader.result)
        return
      }
      reject(new Error('头像格式无效，请选择 JPG 或 PNG 图片'))
    }
    reader.onerror = () => reject(new Error('头像读取失败，请重新选择'))
    reader.readAsDataURL(blob)
  })
}
// #endif

export async function readAvatarAsDataUrl(filePath: string): Promise<string> {
  const normalizedPath = filePath.trim()
  if (!normalizedPath) {
    throw new Error('请先选择头像')
  }
  if (normalizedPath.startsWith('data:image/')) {
    return normalizedPath
  }

  if (typeof uni !== 'undefined' && typeof uni.getFileSystemManager === 'function') {
    const compressed = await compressOnMini(normalizedPath, 80)
    const firstPass = await readMiniFileBase64(compressed)
    if (firstPass.length <= 240_000) {
      return firstPass
    }
    const tighter = await compressOnMini(compressed, 50)
    return readMiniFileBase64(tighter)
  }

  // #ifdef H5
  return readH5BlobAsDataUrl(normalizedPath)
  // #endif
  // #ifndef H5
  throw new Error('当前环境不支持读取头像文件')
  // #endif
}
