function inferMimeType(filePath: string): string {
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
    if (typeof uni.compressImage !== 'function') {
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
        resolve(`data:${inferMimeType(filePath)};base64,${base64}`)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '头像读取失败'))
      },
    })
  })
}

function readBrowserImageAsDataUrl(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.onload = () => {
      try {
        const size = 256
        const canvas = document.createElement('canvas')
        const scale = Math.min(1, size / Math.max(img.width || 1, img.height || 1))
        canvas.width = Math.max(1, Math.round(img.width * scale))
        canvas.height = Math.max(1, Math.round(img.height * scale))
        const ctx = canvas.getContext('2d')
        if (!ctx) {
          reject(new Error('浏览器不支持图片处理'))
          return
        }
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height)
        resolve(canvas.toDataURL('image/jpeg', 0.82))
      } catch {
        reject(new Error('头像读取失败，请重新选择'))
      }
    }
    img.onerror = () => reject(new Error('头像加载失败，请重新选择'))
    img.src = filePath
  })
}

export async function readAvatarAsDataUrl(filePath: string): Promise<string> {
  const normalizedPath = filePath.trim()
  if (!normalizedPath) {
    throw new Error('请先选择头像')
  }
  if (normalizedPath.startsWith('data:image/')) {
    return normalizedPath
  }

  if (typeof uni.getFileSystemManager === 'function') {
    const compressed = await compressOnMini(normalizedPath, 80)
    const firstPass = await readMiniFileBase64(compressed)
    if (firstPass.length <= 240_000) {
      return firstPass
    }
    const tighter = await compressOnMini(compressed, 50)
    return readMiniFileBase64(tighter)
  }

  return readBrowserImageAsDataUrl(normalizedPath)
}
