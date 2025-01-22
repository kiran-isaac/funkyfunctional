import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  base: "http://kiran-isaac.github.io/funkyfunctional/",
  server: {
    fs: {
      strict: false
    },
  },
})
