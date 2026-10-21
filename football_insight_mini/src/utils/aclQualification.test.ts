import { expect, test } from 'bun:test'
import { resolveAclQualification } from './aclQualification'

test('only the top two places carry an AFC Champions League qualification', () => {
  expect(resolveAclQualification(1)?.tier).toBe('elite')
  expect(resolveAclQualification(2)?.tier).toBe('second')
  expect(resolveAclQualification(3)).toBeNull()
  expect(resolveAclQualification(0)).toBeNull()
})
