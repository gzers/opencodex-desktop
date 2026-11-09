import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  build: {
    rolldownOptions: {
      output: {
        codeSplitting: {
          // Vue's barrel re-exports runtime-dom, which calls shared helpers
          // during module evaluation. Keep the framework in one execution unit
          // so production chunk ordering cannot read those helpers before init.
          groups: [{ name: 'vue-framework', test: /node_modules[\\/](?:@vue[\\/]|vue[\\/])/ }],
        },
      },
    },
  },
  server: {
    strictPort: true,
  },
})
