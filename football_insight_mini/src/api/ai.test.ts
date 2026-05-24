import { describe, expect, test } from 'bun:test'

import { streamAiChat } from './ai'

describe('streamAiChat frontend direct mode', () => {
  test('consumes WeChat Cloud textStream and completes normally', async () => {
    ;(globalThis as any).wx = {
      cloud: {
        extend: {
          AI: {
            createModel: () => ({
              streamText: async () => ({
                textStream: (async function* () {
                  yield '你'
                  yield '好'
                })(),
              }),
            }),
          },
        },
      },
    }

    const deltas: string[] = []
    const result = await new Promise<{ reply: string }>((resolve, reject) => {
      streamAiChat(
        {
          message: '你好',
          history: [],
        },
        {
          onDelta: ({ content }) => {
            deltas.push(content)
          },
          onDone: ({ reply }) => {
            resolve({ reply })
          },
          onError: reject,
        },
        { mode: 'frontend_direct' },
      )
    })

    expect(deltas).toEqual(['你', '好'])
    expect(result.reply).toBe('你好')
  })

  test('keeps waiting for done when WeChat Cloud pauses between text chunks', async () => {
    ;(globalThis as any).wx = {
      cloud: {
        extend: {
          AI: {
            createModel: () => ({
              streamText: async () => ({
                textStream: (async function* () {
                  yield '你'
                  await new Promise((resolve) => setTimeout(resolve, 20))
                  yield '好'
                })(),
              }),
            }),
          },
        },
      },
    }

    const deltas: string[] = []
    const result = await new Promise<{ reply: string }>((resolve, reject) => {
      streamAiChat(
        {
          message: '你好',
          history: [],
        },
        {
          onDelta: ({ content }) => {
            deltas.push(content)
          },
          onDone: ({ reply }) => {
            resolve({ reply })
          },
          onError: reject,
        },
        { mode: 'frontend_direct' },
      )
    })

    expect(deltas).toEqual(['你', '好'])
    expect(result.reply).toBe('你好')
  })
})
