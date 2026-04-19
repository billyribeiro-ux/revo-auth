import { existsSync, readFileSync, writeFileSync } from 'node:fs';

const REVO_IMPORT = "import { revoAuth } from '$lib/auth/server';";
const SEQUENCE_IMPORT = "import { sequence } from '@sveltejs/kit/hooks';";
const MARKER = '/* revo-auth:handle */';

export interface MergeHooksResult {
	path: string;
	created: boolean;
	changed: boolean;
	content: string;
}

const FRESH_HOOKS = `${SEQUENCE_IMPORT}
${REVO_IMPORT}
import type { Handle } from '@sveltejs/kit';

${MARKER}
export const handle: Handle = sequence(revoAuth);
`;

/**
 * Merge revoAuth into an existing hooks.server.ts, or create a fresh file.
 *
 * We use a conservative regex/AST-agnostic strategy rather than full magicast
 * manipulation so that we tolerate user formatting. If a `handle` export is
 * found we wrap it; otherwise we append one.
 */
export function mergeHooksServer(path: string): MergeHooksResult {
	if (!existsSync(path)) {
		writeFileSync(path, FRESH_HOOKS, 'utf8');
		return { path, created: true, changed: true, content: FRESH_HOOKS };
	}

	const original = readFileSync(path, 'utf8');
	if (
		original.includes(MARKER) ||
		/from '\$lib\/auth\/server'/.test(original)
	) {
		return { path, created: false, changed: false, content: original };
	}

	let next = original;

	// Ensure imports present
	if (!/from '@sveltejs\/kit\/hooks'/.test(next)) {
		next = `${SEQUENCE_IMPORT}\n${next}`;
	}
	if (!/\$lib\/auth\/server/.test(next)) {
		next = `${REVO_IMPORT}\n${next}`;
	}

	const handleExport =
		/export\s+const\s+handle(?:\s*:\s*[^=]+)?\s*=\s*([^;]+);?/m;
	const match = next.match(handleExport);
	if (match) {
		const existingExpr =
			match[1]?.trim() ?? 'async ({ event, resolve }) => resolve(event)';
		const replacement = `${MARKER}\nexport const handle: Handle = sequence(revoAuth, ${existingExpr});`;
		next = next.replace(handleExport, replacement);
		if (
			!/import\s+type\s*\{[^}]*\bHandle\b[^}]*\}\s*from\s*'@sveltejs\/kit'/.test(
				next,
			)
		) {
			next = `import type { Handle } from '@sveltejs/kit';\n${next}`;
		}
	} else {
		const hasHandleType =
			/import\s+type\s*\{[^}]*\bHandle\b[^}]*\}\s*from\s*'@sveltejs\/kit'/.test(
				next,
			);
		const trailer = `\n${MARKER}\n${hasHandleType ? '' : "import type { Handle } from '@sveltejs/kit';\n"}export const handle: Handle = sequence(revoAuth);\n`;
		next = `${next.replace(/\s*$/, '')}\n${trailer}`;
	}

	writeFileSync(path, next, 'utf8');
	return { path, created: false, changed: true, content: next };
}
