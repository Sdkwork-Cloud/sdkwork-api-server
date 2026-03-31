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

  if (id.includes('\\motion\\') || id.includes('/motion/')) {
    return 'motion-vendor';
  }

  return undefined;
}

export default defineConfig({
  base: '/admin/',
  plugins: [react(), tailwindcss()],
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },
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
