import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

function manualChunks(id: string) {
  if (!id.includes('node_modules')) {
    return undefined;
  }

  if (
    id.includes('\\react\\')
    || id.includes('/react/')
    || id.includes('\\react-dom\\')
    || id.includes('/react-dom/')
    || id.includes('\\react-router')
    || id.includes('/react-router')
    || id.includes('\\scheduler\\')
    || id.includes('/scheduler/')
    || id.includes('\\@remix-run\\router\\')
    || id.includes('/@remix-run/router/')
  ) {
    return 'react-vendor';
  }

  if (id.includes('\\@radix-ui\\') || id.includes('/@radix-ui/')) {
    return 'radix-vendor';
  }

  if (id.includes('\\lucide-react\\') || id.includes('/lucide-react/')) {
    return 'icon-vendor';
  }

  if (id.includes('\\recharts\\') || id.includes('/recharts/')) {
    return 'charts-vendor';
  }

  return undefined;
}

export default defineConfig({
  base: '/portal/',
  plugins: [react(), tailwindcss()],
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },
  server: {
    host: '0.0.0.0',
    port: 5174,
    strictPort: true,
    proxy: {
      '/api/portal': {
        target: 'http://127.0.0.1:8082',
        changeOrigin: true,
        rewrite: (sourcePath) => sourcePath.replace(/^\/api\/portal/, '/portal'),
      },
    },
  },
  preview: {
    host: '0.0.0.0',
    port: 4174,
    strictPort: true,
  },
});
