import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  // Tauri dev server: use port 5173, allow all origins
  server: {
    port: 5173,
    strictPort: true,
  },
  // Prevent Vite from clearing the terminal on dev start
  clearScreen: false,
  // Produce relative asset paths for Tauri file:// serving
  base: './',
})
