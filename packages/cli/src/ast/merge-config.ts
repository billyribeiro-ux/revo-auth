import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { parseModule, type ProxifiedModule } from 'magicast';

export interface ConfigMergeResult {
	path: string;
	created: boolean;
	changed: boolean;
	added: string[];
	conflicts: string[];
}

/**
 * Scalar/array defaults to add into `auth.config.ts`. Nested objects are
 * merged shallowly (existing keys never overwritten).
 */
export type DefaultsMap = Record<string, unknown>;

function hasKey(mod: ProxifiedModule<Record<string, unknown>>, key: string): boolean {
	const exports = mod.exports as { default?: Record<string, unknown> };
	const cfg = exports.default;
	if (!cfg || typeof cfg !== 'object') return false;
	return Object.prototype.hasOwnProperty.call(cfg, key);
}

/**
 * Create `auth.config.ts` from scratch.
 */
export function writeFreshConfig(path: string, defaults: DefaultsMap): ConfigMergeResult {
	const body = `import { defineAuthConfig } from '@revo-auth/sdk-sveltekit';

export default defineAuthConfig(${JSON.stringify(defaults, null, 2)});
`;
	writeFileSync(path, body, 'utf8');
	return {
		path,
		created: true,
		changed: true,
		added: Object.keys(defaults),
		conflicts: [],
	};
}

/**
 * Merge new default keys into an existing `auth.config.ts`. Never overwrites
 * existing values. If a collision is detected at the type-level (e.g. user
 * has an incompatible shape), writes a `.orig` sidecar with the template
 * and reports the conflict.
 */
export function mergeAuthConfig(path: string, defaults: DefaultsMap): ConfigMergeResult {
	if (!existsSync(path)) {
		return writeFreshConfig(path, defaults);
	}
	const source = readFileSync(path, 'utf8');
	let mod: ProxifiedModule<Record<string, unknown>>;
	try {
		mod = parseModule(source);
	} catch (err) {
		const origPath = `${path}.orig`;
		writeFileSync(origPath, source, 'utf8');
		return {
			path,
			created: false,
			changed: false,
			added: [],
			conflicts: [
				`Could not parse ${path}: ${err instanceof Error ? err.message : String(err)}. Wrote backup to ${origPath}.`,
			],
		};
	}

	const exports = mod.exports as { default?: Record<string, unknown> };
	if (!exports.default) {
		const origPath = `${path}.orig`;
		writeFileSync(origPath, source, 'utf8');
		return {
			path,
			created: false,
			changed: false,
			added: [],
			conflicts: [
				`auth.config.ts has no default export; cannot merge. Wrote backup to ${origPath}.`,
			],
		};
	}

	const added: string[] = [];
	const cfg = exports.default as Record<string, unknown>;
	for (const [key, value] of Object.entries(defaults)) {
		if (hasKey(mod, key)) continue;
		cfg[key] = value;
		added.push(key);
	}

	if (added.length === 0) {
		return { path, created: false, changed: false, added: [], conflicts: [] };
	}

	const output = mod.generate().code;
	writeFileSync(path, output, 'utf8');
	return { path, created: false, changed: true, added, conflicts: [] };
}
