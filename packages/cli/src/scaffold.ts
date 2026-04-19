import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'pathe';
import type { Manifest } from './templates/manifest.js';
import { setManifestEntry } from './templates/manifest.js';
import {
	type RenderContext,
	hashContent,
	renderTemplate,
} from './templates/render.js';

/**
 * Locate the templates directory that ships with this package.
 *
 * After tsup build, this file lives in `packages/cli/dist/bin.js`; templates
 * live at `packages/cli/templates/`. In dev (tsx), it lives in
 * `packages/cli/src/scaffold.ts`. We walk up until we find a sibling
 * `templates/` directory.
 */
export function findTemplatesRoot(): string {
	const here = dirname(fileURLToPath(import.meta.url));
	let current = here;
	for (let i = 0; i < 5; i++) {
		const candidate = resolve(current, 'templates');
		if (existsSync(candidate)) return candidate;
		const parent = resolve(current, '..');
		if (parent === current) break;
		current = parent;
	}
	throw new Error(
		`Could not locate @revo-auth/cli templates directory (searched from ${here}).`,
	);
}

export interface ScaffoldFile {
	/** Template path relative to the templates root (e.g. "sveltekit/lib/client.ts.hbs"). */
	template: string;
	/** Output path relative to the project root (e.g. "src/lib/auth/client.ts"). */
	outRel: string;
	/** If the output file exists and was not produced by us, skip. */
	skipIfUnmanagedExists?: boolean;
}

export interface ScaffoldResult {
	written: string[];
	skipped: string[];
}

export function scaffoldFiles(
	cwd: string,
	templatesRoot: string,
	files: ScaffoldFile[],
	context: RenderContext,
	manifest: Manifest,
): ScaffoldResult {
	const written: string[] = [];
	const skipped: string[] = [];

	for (const file of files) {
		const templatePath = resolve(templatesRoot, file.template);
		const outAbs = resolve(cwd, file.outRel);
		const rendered = renderTemplate(templatePath, context);
		const templateHash = hashContent(readFileSync(templatePath, 'utf8'));

		if (
			existsSync(outAbs) &&
			file.skipIfUnmanagedExists &&
			!manifest.files[file.outRel]
		) {
			skipped.push(file.outRel);
			continue;
		}

		const dir = dirname(outAbs);
		if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
		writeFileSync(outAbs, rendered, 'utf8');

		setManifestEntry(manifest, file.outRel, {
			templateHash,
			writtenHash: hashContent(rendered),
			template: file.template,
		});
		written.push(file.outRel);
	}

	return { written, skipped };
}

export function appendGitignore(path: string, entries: string[]): string[] {
	const existing = existsSync(path) ? readFileSync(path, 'utf8') : '';
	const lines = existing.split(/\r?\n/);
	const present = new Set(lines.map((l) => l.trim()));
	const added: string[] = [];
	for (const entry of entries) {
		if (!present.has(entry)) {
			lines.push(entry);
			added.push(entry);
		}
	}
	if (added.length === 0) return [];
	const content = `${lines.join('\n').replace(/\n+$/, '')}\n`;
	writeFileSync(path, content, 'utf8');
	return added;
}
