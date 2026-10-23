import path from 'node:path'
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import process from "node:process";
import tailwindcss from '@tailwindcss/vite'
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [vue(),  tailwindcss()],
    resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },

  build: {
    // 单块超过这个值才提示，正常的 vendor 块不算问题
    chunkSizeWarningLimit: 900,
    rollupOptions: {
      output: {
        entryFileNames: 'assets/js/[name]-[hash].js',
        chunkFileNames: 'assets/js/[name]-[hash].js',
        assetFileNames: (asset) => {
          const name = asset.names?.[0] ?? asset.name ?? '';
          if (name.endsWith('.css')) return 'assets/css/[name]-[hash][extname]';
          if (/\.(woff2?|ttf|otf|eot)$/i.test(name)) return 'assets/fonts/[name]-[hash][extname]';
          if (/\.(png|jpe?g|gif|svg|webp|avif|ico)$/i.test(name)) return 'assets/img/[name]-[hash][extname]';
          return 'assets/[name]-[hash][extname]';
        },
        // 把几个大的第三方依赖拆开，首屏只加载必需的
        manualChunks(id) {
          if (!id.includes('node_modules')) return undefined;
          if (id.includes('/vue/') || id.includes('/@vue/') || id.includes('pinia')) return 'vendor-vue';
          if (id.includes('reka-ui') || id.includes('@lucide') || id.includes('/lucide')) return 'vendor-ui';
          if (id.includes('@fontsource')) return 'vendor-fonts';
          return 'vendor';
        },
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
