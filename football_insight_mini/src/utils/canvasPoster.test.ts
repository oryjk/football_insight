import { expect, test } from 'bun:test'
import { savePosterWithPermission } from './canvasPoster'

test('switching context while permission lookup is pending cannot save an old image', async () => {
  let resolveSetting!: (result: unknown) => void
  let current = true, saved = false
  const platform = {
    getSetting: (options: any) => { resolveSetting = options.success },
    showModal: () => {}, openSetting: () => {}, showToast: () => {},
    saveImageToPhotosAlbum: () => { saved = true },
  }
  const saving = savePosterWithPermission('old.png', platform as any, () => current)
  current = false
  resolveSetting({ authSetting: {} })
  await saving
  expect(saved).toBe(false)
})
