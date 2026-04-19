import { existsSync, readFileSync } from 'node:fs';
import { defineCommand } from 'citty';
import { ofetch } from 'ofetch';
import { resolve } from 'pathe';
import { resolveProjectPaths } from '../detect/paths.js';
import {
	info,
	intro,
	error as logError,
	outro,
	success,
	warn,
} from '../prompts.js';
import { readManifest } from '../templates/manifest.js';
import { hashContent } from '../templates/render.js';

const REQUIRED_ENV = [
	'REVO_AUTH_SERVER_URL',
	'REVO_AUTH_APP_ID',
	'REVO_AUTH_PUBLIC_KEY',
	'REVO_AUTH_SECRET_KEY',
];

function parseEnvFile(path: string): Map<string, string> {
	const map = new Map<string, string>();
	if (!existsSync(path)) return map;
	const src = readFileSync(path, 'utf8');
	for (const line of src.split(/\r?\n/)) {
		const trimmed = line.trim();
		if (!trimmed || trimmed.startsWith('#')) continue;
		const eq = trimmed.indexOf('=');
		if (eq === -1) continue;
		map.set(trimmed.slice(0, eq).trim(), trimmed.slice(eq + 1).trim());
	}
	return map;
}

export const doctorCommand = defineCommand({
	meta: { name: 'doctor', description: 'Diagnose your Revo-Auth setup' },
	args: {
		cwd: { type: 'string', default: process.cwd() },
	},
	async run({ args }) {
		const cwd = args.cwd;
		intro('Revo-Auth doctor');
		const paths = resolveProjectPaths(cwd);
		let problems = 0;

		// .env
		const env = parseEnvFile(paths.envFile);
		for (const key of REQUIRED_ENV) {
			const v = env.get(key);
			if (!v || v.startsWith('replace-me')) {
				logError(`.env missing or placeholder for ${key}`);
				problems++;
			} else {
				success(`.env has ${key}`);
			}
		}

		// hooks.server.ts
		if (!existsSync(paths.hooksServer)) {
			logError('src/hooks.server.ts is missing');
			problems++;
		} else {
			const src = readFileSync(paths.hooksServer, 'utf8');
			if (
				/\$lib\/auth\/server/.test(src) &&
				/sequence\s*\(\s*revoAuth/.test(src)
			) {
				success('hooks.server.ts wired');
			} else {
				logError('hooks.server.ts does not reference revoAuth in sequence()');
				problems++;
			}
		}

		// manifest vs filesystem
		const manifest = readManifest(paths.manifestFile);
		if (!manifest) {
			warn('No manifest found — run `revo-auth init`.');
			problems++;
		} else {
			for (const [relPath, entry] of Object.entries(manifest.files)) {
				const abs = resolve(cwd, relPath);
				if (!existsSync(abs)) {
					warn(`Manifest references missing file: ${relPath}`);
					continue;
				}
				const actual = hashContent(readFileSync(abs, 'utf8'));
				if (actual !== entry.writtenHash) {
					info(`user-modified: ${relPath}`);
				} else {
					success(`unchanged: ${relPath}`);
				}
			}
		}

		// server health
		const serverUrl =
			env.get('REVO_AUTH_SERVER_URL') ?? process.env.REVO_AUTH_SERVER_URL;
		if (serverUrl) {
			try {
				await ofetch.raw('/health', {
					baseURL: serverUrl,
					method: 'HEAD',
					timeout: 5000,
				});
				success(`server reachable at ${serverUrl}`);
			} catch (err) {
				warn(
					`server ${serverUrl} is not reachable: ${err instanceof Error ? err.message : String(err)}`,
				);
			}
		}

		if (problems === 0) outro('Everything looks good.');
		else {
			outro(`${problems} problem(s) found.`);
			process.exitCode = 1;
		}
	},
});
