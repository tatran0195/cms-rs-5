import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm'],
  platform: 'browser',
  clean: true,
  dts: true,
  sourcemap: true,
  suppressWarnings: [/module level directive/i],
  deps: {
    neverBundle: ['@cms/ui', '@cms/icons'],
  },
});
