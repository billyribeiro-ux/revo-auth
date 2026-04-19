import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { defineCommand } from 'citty';
import { dirname, resolve } from 'pathe';
import { resolveProjectPaths } from '../detect/paths.js';
import {
	info,
	intro,
	error as logError,
	outro,
	success,
	warn,
} from '../prompts.js';

const COMPONENTS = [
	'login-form',
	'signup-form',
	'oauth-button',
	'passkey-button',
	'magic-link-form',
	'mfa-setup',
	'mfa-challenge',
	'session-list',
	'account-linking',
	'password-field',
	'password-strength-meter',
	'verify-email-banner',
] as const;

type Component = (typeof COMPONENTS)[number];

function toPascal(name: string): string {
	return name
		.split('-')
		.map((s) => (s.length > 0 ? (s[0] ?? '').toUpperCase() + s.slice(1) : s))
		.join('');
}

function resolveUiPackagePath(cwd: string): string | null {
	try {
		const require = createRequire(resolve(cwd, 'package.json'));
		const pkgJsonPath = require.resolve('@revo-auth/ui-sveltekit/package.json');
		return dirname(pkgJsonPath);
	} catch {
		return null;
	}
}

export const uiCommand = defineCommand({
	meta: { name: 'ui', description: 'Copy UI components into your project' },
	args: {
		components: {
			type: 'positional',
			required: true,
			description: `Components (comma or space separated): ${COMPONENTS.join(', ')}`,
		},
		cwd: { type: 'string', default: process.cwd() },
	},
	async run({ args, rawArgs }) {
		const cwd = args.cwd;
		intro('Revo-Auth ui');

		const requested = new Set<string>();
		const list = [args.components, ...(rawArgs ?? [])].flatMap((v) =>
			typeof v === 'string' ? v.split(/[,\s]+/).filter(Boolean) : [],
		);
		for (const entry of list) requested.add(entry);

		const invalid = [...requested].filter(
			(c) => !COMPONENTS.includes(c as Component),
		);
		if (invalid.length > 0) {
			logError(`Unknown components: ${invalid.join(', ')}`);
			info(`Available: ${COMPONENTS.join(', ')}`);
			process.exitCode = 1;
			return;
		}

		const uiPath = resolveUiPackagePath(cwd);
		if (!uiPath) {
			logError('Could not resolve @revo-auth/ui-sveltekit — install it first.');
			process.exitCode = 1;
			return;
		}

		const paths = resolveProjectPaths(cwd);
		if (!existsSync(paths.authComponentsDir))
			mkdirSync(paths.authComponentsDir, { recursive: true });

		for (const comp of requested) {
			const pascal = toPascal(comp);
			const candidates = [
				resolve(uiPath, 'src/components', `${pascal}.svelte`),
				resolve(uiPath, 'dist/components', `${pascal}.svelte`),
			];
			const source = candidates.find((p) => existsSync(p));
			if (!source) {
				warn(`${pascal}.svelte not found in @revo-auth/ui-sveltekit; skipping`);
				continue;
			}
			const content = readFileSync(source, 'utf8');
			const rewritten = content.replace(
				/from '@revo-auth\/ui-sveltekit(\/[^']*)?'/g,
				"from '$lib/auth/components'",
			);
			const outPath = resolve(paths.authComponentsDir, `${pascal}.svelte`);
			writeFileSync(outPath, rewritten, 'utf8');
			success(`copied ${pascal}.svelte`);
		}

		// Write barrel if missing
		const barrel = resolve(paths.authComponentsDir, 'index.ts');
		if (!existsSync(barrel)) {
			const exports = [...requested]
				.map(
					(c) =>
						`export { default as ${toPascal(c)} } from './${toPascal(c)}.svelte';`,
				)
				.join('\n');
			writeFileSync(barrel, `${exports}\n`, 'utf8');
			success('wrote components/index.ts');
		}
		outro('Done.');
	},
});
