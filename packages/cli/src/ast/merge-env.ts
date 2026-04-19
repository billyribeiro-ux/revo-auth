import { existsSync, readFileSync, writeFileSync } from 'node:fs';

export interface EnvVar {
	key: string;
	value: string;
	comment?: string;
}

function parseEnv(source: string): { keys: Set<string>; lines: string[] } {
	const lines = source === '' ? [] : source.split(/\r?\n/);
	const keys = new Set<string>();
	for (const line of lines) {
		const trimmed = line.trim();
		if (!trimmed || trimmed.startsWith('#')) continue;
		const eq = trimmed.indexOf('=');
		if (eq === -1) continue;
		keys.add(trimmed.slice(0, eq).trim());
	}
	return { keys, lines };
}

export interface MergeEnvResult {
	path: string;
	added: string[];
	skipped: string[];
	content: string;
}

/**
 * Append env vars to a dotenv-style file. Creates the file if absent.
 * Idempotent: keys that already exist are left alone.
 */
export function mergeEnvFile(path: string, entries: EnvVar[]): MergeEnvResult {
	const existing = existsSync(path) ? readFileSync(path, 'utf8') : '';
	const { keys, lines } = parseEnv(existing);
	const added: string[] = [];
	const skipped: string[] = [];

	const needsTrailingNewline = existing.length > 0 && !existing.endsWith('\n');
	const output: string[] = [...lines];
	if (needsTrailingNewline) {
		// ensure blank separator before appending
		output.push('');
	}
	const startLen = output.length;
	if (existing.length > 0 && output[output.length - 1] !== '') {
		output.push('');
	}
	output.push('# Added by @revo-auth/cli');

	for (const entry of entries) {
		if (keys.has(entry.key)) {
			skipped.push(entry.key);
			continue;
		}
		if (entry.comment) output.push(`# ${entry.comment}`);
		output.push(`${entry.key}=${entry.value}`);
		added.push(entry.key);
	}

	if (added.length === 0) {
		// Roll back header insertion if nothing added.
		output.length = startLen;
	}

	const content = `${output.join('\n').replace(/\n+$/, '')}\n`;
	writeFileSync(path, content, 'utf8');
	return { path, added, skipped, content };
}
