import { defineConfig } from 'vite';
import solid from '@solidjs/vite-plugin';
export default defineConfig({ plugins: [solid()], build: { target: 'esnext' }, server: { host: '127.0.0.1', port: 5174, strictPort: true, fs: { allow: [".."] } } });
