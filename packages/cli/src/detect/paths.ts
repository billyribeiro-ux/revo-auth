import { existsSync } from 'node:fs';
import { resolve } from 'pathe';

export interface ProjectPaths {
	cwd: string;
	src: string;
	lib: string;
	routes: string;
	hooksServer: string;
	envFile: string;
	envExample: string;
	gitignore: string;
	manifestDir: string;
	manifestFile: string;
	authConfig: string;
	authLibDir: string;
	authRoutesDir: string;
	authComponentsDir: string;
}

export function resolveProjectPaths(cwd: string): ProjectPaths {
	const src = resolve(cwd, 'src');
	const lib = resolve(src, 'lib');
	const routes = resolve(src, 'routes');
	return {
		cwd,
		src,
		lib,
		routes,
		hooksServer: resolve(src, 'hooks.server.ts'),
		envFile: resolve(cwd, '.env'),
		envExample: resolve(cwd, '.env.example'),
		gitignore: resolve(cwd, '.gitignore'),
		manifestDir: resolve(cwd, '.revo-auth'),
		manifestFile: resolve(cwd, '.revo-auth', 'manifest.json'),
		authConfig: resolve(cwd, 'auth.config.ts'),
		authLibDir: resolve(lib, 'auth'),
		authRoutesDir: resolve(routes, 'auth'),
		authComponentsDir: resolve(lib, 'auth', 'components'),
	};
}

export function pathExists(path: string): boolean {
	return existsSync(path);
}
