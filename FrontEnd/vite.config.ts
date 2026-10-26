import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(import.meta.dirname, './src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: true,
    watch: {
      ignored: ['**/../Backend/**'],
    },
  },
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
})