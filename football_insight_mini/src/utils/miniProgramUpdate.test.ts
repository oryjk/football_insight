import { describe, expect, test } from 'bun:test'
import { setupMiniProgramUpdatePrompt, type UpdatePromptDeps } from './miniProgramUpdate'

function fakeDeps(confirm: boolean) {
  const handlers: { ready?: (result: unknown) => void, failed?: (result: unknown) => void } = {}
  let applied = 0
  let prompts = 0
  const deps: UpdatePromptDeps = {
    manager: {
      onUpdateReady: (cb) => { handlers.ready = cb },
      onUpdateFailed: (cb) => { handlers.failed = cb },
      applyUpdate: () => { applied += 1 },
    },
    confirmRestart: () => {
      prompts += 1
      return Promise.resolve(confirm)
    },
  }
  return { deps, handlers, applied: () => applied, prompts: () => prompts }
}

describe('setupMiniProgramUpdatePrompt', () => {
  test('新版本下载完成后询问，确认则重启到新版本', async () => {
    const fake = fakeDeps(true)
    setupMiniProgramUpdatePrompt(fake.deps)

    fake.handlers.ready?.({})
    await Promise.resolve()

    expect(fake.prompts()).toBe(1)
    expect(fake.applied()).toBe(1)
  })

  test('用户取消则不重启，下次冷启动自动生效', async () => {
    const fake = fakeDeps(false)
    setupMiniProgramUpdatePrompt(fake.deps)

    fake.handlers.ready?.({})
    await Promise.resolve()

    expect(fake.prompts()).toBe(1)
    expect(fake.applied()).toBe(0)
  })

  test('下载失败静默，不打扰用户', () => {
    const fake = fakeDeps(true)
    setupMiniProgramUpdatePrompt(fake.deps)

    fake.handlers.failed?.({})

    expect(fake.prompts()).toBe(0)
    expect(fake.applied()).toBe(0)
  })
})
