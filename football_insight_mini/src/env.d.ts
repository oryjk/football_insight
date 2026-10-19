/// <reference types="vite/client" />

declare const __FOOTBALL_BUILD_INFO__: { revision: string; builtAt: string }

declare module '*.vue' {
  import { DefineComponent } from 'vue'
  // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/ban-types
  const component: DefineComponent<{}, {}, any>
  export default component
}
