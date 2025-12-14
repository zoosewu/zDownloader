import { defineConfig } from 'vite';

/**
 * Vite plugin to remove ES module export syntax
 *
 * Zoo Downloader's module loader uses deno_core's execute_script() API,
 * which doesn't support ES module syntax (import/export).
 *
 * This plugin automatically transforms:
 *   export class Foo {}        → class Foo {}
 *   export default class Foo   → class Foo
 *   export function bar()      → function bar()
 *   export const x = 1         → const x = 1
 *
 * This allows developers to write standard ES modules during development,
 * while producing compatible output for the runtime.
 */
function removeExportSyntax() {
  return {
    name: 'remove-export-syntax',

    // Apply transformation after code generation
    generateBundle(options, bundle) {
      for (const fileName in bundle) {
        const chunk = bundle[fileName];

        if (chunk.type === 'chunk') {
          // Add banner at the top
          const banner = '// K2S Module - Compiled with Vite\n// Export syntax removed for Zoo Downloader compatibility\n\n';

          // Transform export statements to plain declarations
          let code = chunk.code
            // Remove export { ... } blocks
            .replace(/export\s*\{[^}]*\}\s*;?\s*/g, '')
            // Remove inline export declarations
            .replace(/export default class\s+/g, 'class ')
            .replace(/export class\s+/g, 'class ')
            .replace(/export default function\s+/g, 'function ')
            .replace(/export function\s+/g, 'function ')
            .replace(/export default const\s+/g, 'const ')
            .replace(/export const\s+/g, 'const ')
            .replace(/export default let\s+/g, 'let ')
            .replace(/export let\s+/g, 'let ')
            .replace(/export default var\s+/g, 'var ')
            .replace(/export var\s+/g, 'var ')
            .replace(/export default\s+/g, '');

          // Add banner and update chunk
          chunk.code = banner + code;
        }
      }
    }
  };
}

export default defineConfig({
  build: {
    // Output directory for compiled code
    outDir: 'dist',

    // Library mode - build as a library
    lib: {
      entry: 'src/index.js',
      formats: ['es'],  // ES module format (will be transformed by plugin)
      fileName: () => 'index.js',
    },

    // Rollup options
    rollupOptions: {
      output: {
        // Disable code splitting
        inlineDynamicImports: true,

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

  // Register the custom plugin
  plugins: [
    removeExportSyntax(),
  ],
});
