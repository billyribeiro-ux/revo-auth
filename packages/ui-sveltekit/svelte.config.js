import { vitePreprocess } from '@sveltejs/package/preprocess';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	compilerOptions: {
		runes: true
	}
};

export default config;
