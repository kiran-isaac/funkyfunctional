import { defineConfig, searchForWorkspaceRoot } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        // all wasm
        process.cwd() + "/../wasm_lib/pkg/sfl_wasm_lib_bg.wasm",
      ],
    },
  },
})
