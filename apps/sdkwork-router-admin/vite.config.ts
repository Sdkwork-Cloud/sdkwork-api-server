import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  base: '/admin/',
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      'sdkwork-router-admin-apirouter': new URL(
        './packages/sdkwork-router-admin-apirouter/src/index.ts',
        import.meta.url,
      ).pathname,
    },
  },
  server: {
    host: '0.0.0.0',
    proxy: {
      '/api/admin': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/admin/, '/admin'),
      },
    },
  },
  preview: {
    host: '0.0.0.0',
    port: 4173,
    strictPort: true,
  },
});
