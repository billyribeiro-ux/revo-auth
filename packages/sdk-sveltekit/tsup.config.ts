import { defineConfig } from 'tsup';

export default defineConfig({
	entry: ['src/index.ts', 'src/client.svelte.ts'],
	format: ['esm'],
	target: 'esnext',
	dts: true,
	clean: true,
	sourcemap: true,
	treeshake: true,
	splitting: false,
	outDir: 'dist',
	external: ['svelte', '@sveltejs/kit'],
});
