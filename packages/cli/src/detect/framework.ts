import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'pathe';

export interface FrameworkInfo {
	framework: 'sveltekit' | 'unknown';
	sveltekitVersion: string | null;
	sveltekitMajor: number | null;
	svelteVersion: string | null;
	hasTypeScript: boolean;
	hasTsConfig: boolean;
	nodeVersion: string;
	packageJsonPath: string;
}

export interface FrameworkCheck {
	ok: boolean;
	info: FrameworkInfo;
	errors: string[];
}

interface PackageJson {
	name?: string;
	dependencies?: Record<string, string>;
	devDependencies?: Record<string, string>;
	engines?: { node?: string };
}

function readPackageJson(path: string): PackageJson {
	const raw = readFileSync(path, 'utf8');
	return JSON.parse(raw) as PackageJson;
}

function coerceMajor(version: string | null): number | null {
	if (!version) return null;
	const match = /(\d+)/.exec(version.replace(/^[\^~>=<]+/, ''));
	return match?.[1] ? Number.parseInt(match[1], 10) : null;
}

export function detectFramework(cwd: string): FrameworkCheck {
	const packageJsonPath = resolve(cwd, 'package.json');
	const errors: string[] = [];

	if (!existsSync(packageJsonPath)) {
		return {
			ok: false,
			errors: [`No package.json found at ${packageJsonPath}`],
			info: {
				framework: 'unknown',
				sveltekitVersion: null,
				sveltekitMajor: null,
				svelteVersion: null,
				hasTypeScript: false,
				hasTsConfig: false,
				nodeVersion: process.version,
				packageJsonPath,
			},
		};
	}

	const pkg = readPackageJson(packageJsonPath);
	const deps: Record<string, string> = {
		...(pkg.dependencies ?? {}),
		...(pkg.devDependencies ?? {}),
	};

	const sveltekitVersion = deps['@sveltejs/kit'] ?? null;
	const svelteVersion = deps.svelte ?? null;
	const sveltekitMajor = coerceMajor(sveltekitVersion);
	const hasTypeScript = Boolean(deps.typescript);
	const hasTsConfig =
		existsSync(resolve(cwd, 'tsconfig.json')) ||
		existsSync(resolve(cwd, 'jsconfig.json'));

	const framework: FrameworkInfo['framework'] = sveltekitVersion
		? 'sveltekit'
		: 'unknown';

	if (framework !== 'sveltekit') {
		errors.push(
			'SvelteKit is required. Could not find @sveltejs/kit in dependencies.',
		);
	}
	if (sveltekitMajor !== null && sveltekitMajor < 2) {
		errors.push(
			`SvelteKit >= 2.0 required (found ${sveltekitVersion}). Upgrade with \`pnpm add -D @sveltejs/kit@latest\`.`,
		);
	}
	if (!hasTypeScript) {
		errors.push(
			'TypeScript is required. Run `pnpm add -D typescript` and create a tsconfig.json.',
		);
	}

	return {
		ok: errors.length === 0,
		errors,
		info: {
			framework,
			sveltekitVersion,
			sveltekitMajor,
			svelteVersion,
			hasTypeScript,
			hasTsConfig,
			nodeVersion: process.version,
			packageJsonPath,
		},
	};
}
