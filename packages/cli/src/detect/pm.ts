import { existsSync } from 'node:fs';
import { resolve } from 'pathe';

export interface PackageManagerCheck {
	ok: boolean;
	message: string;
	foreignLocks: string[];
}

/**
 * Enforce pnpm. Errors out (via returned flag) if a foreign lockfile exists.
 */
export function detectPackageManager(cwd: string): PackageManagerCheck {
	const foreignLocks: string[] = [];
	const candidates = ['package-lock.json', 'yarn.lock', 'bun.lockb', 'bun.lock'];
	for (const name of candidates) {
		if (existsSync(resolve(cwd, name))) {
			foreignLocks.push(name);
		}
	}
	if (foreignLocks.length > 0) {
		return {
			ok: false,
			foreignLocks,
			message: [
				'Revo-Auth requires pnpm.',
				`Detected foreign lockfile(s): ${foreignLocks.join(', ')}.`,
				`Delete them and run \`pnpm import\` then \`pnpm install\`.`,
			].join('\n'),
		};
	}
	return { ok: true, foreignLocks: [], message: 'pnpm detected' };
}

export function assertPnpm(cwd: string): void {
	const result = detectPackageManager(cwd);
	if (!result.ok) {
		throw new Error(result.message);
	}
}
