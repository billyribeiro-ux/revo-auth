import { defineConfig } from 'tsup';

export default defineConfig({
	entry: ['src/bin.ts'],
	format: ['esm'],
	target: 'node22',
	dts: true,
	clean: true,
	sourcemap: true,
	treeshake: true,
	splitting: false,
	outDir: 'dist',
	banner: {
		js: '#!/usr/bin/env node',
	},
});
