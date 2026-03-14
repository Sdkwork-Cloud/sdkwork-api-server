import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/admin': {
        target: process.env.SDKWORK_ADMIN_PROXY_TARGET ?? 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
      '/portal': {
        target: process.env.SDKWORK_PORTAL_PROXY_TARGET ?? 'http://127.0.0.1:8082',
        changeOrigin: true,
      },
      '/v1': {
        target: process.env.SDKWORK_GATEWAY_PROXY_TARGET ?? 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
});
