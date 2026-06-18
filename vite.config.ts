import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import path from 'node:path';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: {
    alias: {
      $lib: path.resolve('./src/lib'),
      '@': path.resolve('./src')
    }
  },
  build: {
    outDir: 'build',
    emptyOutDir: true
  },
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true
  }
});
