/**
 * 小程序版本更新提示：冷启动时微信检查到新版本并下载完成后，询问用户是否立即重启到新版本。
 * 取消则沿用当前版本，下次冷启动自动生效；下载失败静默，下次冷启动微信会重试。
 * 只在小程序端调用（H5 无 UpdateManager），调用方负责 #ifdef MP-WEIXIN 隔离。
 */
export interface UpdatePromptDeps {
  manager: Pick<UniNamespace.UpdateManager, 'onUpdateReady' | 'onUpdateFailed' | 'applyUpdate'>
  confirmRestart: () => Promise<boolean>
}

function confirmRestartByModal(): Promise<boolean> {
  return new Promise((resolve) => {
    uni.showModal({
      title: '发现新版本',
      content: '新版本已经准备好，是否重启小程序使用新版本？',
      confirmText: '立即重启',
      cancelText: '稍后',
      success: ({ confirm }) => resolve(confirm),
      fail: () => resolve(false),
    })
  })
}

export function setupMiniProgramUpdatePrompt(deps?: UpdatePromptDeps): void {
  const { manager, confirmRestart } = deps ?? {
    manager: uni.getUpdateManager(),
    confirmRestart: confirmRestartByModal,
  }

  manager.onUpdateReady(() => {
    void confirmRestart().then((confirmed) => {
      if (confirmed) manager.applyUpdate()
    })
  })
  manager.onUpdateFailed(() => {
    console.warn('[mini update] 新版本下载失败，下次冷启动重试')
  })
}
