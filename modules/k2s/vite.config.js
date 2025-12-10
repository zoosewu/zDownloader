import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    // Output directory for compiled code
    outDir: 'dist',

    // Library mode - build as a library
    lib: {
      entry: 'src/index.js',
      formats: ['es'],  // ES module format
      fileName: () => 'index.js',
    },

    // Rollup options
    rollupOptions: {
      output: {
        // Disable code splitting
        inlineDynamicImports: true,

        // Banner to add at the top of the file
        banner: '// K2S Module - Compiled with Vite\n// Note: ES module syntax (export) will be automatically transformed by loader\n',

        // Preserve module structure
        preserveModules: false,
      },
    },

    // Minification: disable for better debugging
    // Change to 'terser' or 'esbuild' for production
    minify: false,

    // Source maps for debugging
    sourcemap: false,

    // Target modern browsers/environments
    target: 'es2020',
  },
});
