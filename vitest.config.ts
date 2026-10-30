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
    // @mui/icons-material v7 ESM directory-imports @mui/material/utils (v6); inline so the alias below applies
    server: { deps: { inline: [/@mui\//] } },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@mui/material/utils': '@mui/material/node/utils/index.js',
      '@mui/system/colorManipulator': '@mui/system/node/colorManipulator/index.js',
    },
  },
});
