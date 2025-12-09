import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/__tests__/setup.ts'],
    include: ['src/**/__tests__/**/*.test.{ts,tsx}', 'src/**/*.test.{ts,tsx}'],
    exclude: ['node_modules', 'dist', 'src-tauri'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@mui/material/utils': '@mui/material/node/utils/index.js',
      '@mui/system/colorManipulator': '@mui/system/node/colorManipulator/index.js',
    },
  },
});
