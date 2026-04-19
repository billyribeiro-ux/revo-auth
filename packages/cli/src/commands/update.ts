import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { defineCommand } from 'citty';
import { resolve } from 'pathe';
import { resolveProjectPaths } from '../detect/paths.js';
import { merge3 } from '../diff.js';
import {
	error as logError,
	info,
	intro,
	note,
	outro,
	success,
	warn,
} from '../prompts.js';
import { findTemplatesRoot } from '../scaffold.js';
import { readManifest, writeManifest, type Manifest } from '../templates/manifest.js';
import { hashContent, renderTemplate } from '../templates/render.js';

const CLI_VERSION = '0.1.0';

export const updateCommand = defineCommand({
	meta: { name: 'update', description: 'Update scaffolded files to the latest templates' },
	args: {
		interactive: { type: 'boolean', default: false },
		auto: { type: 'boolean', default: false },
		cwd: { type: 'string', description: 'Project root', default: process.cwd() },
	},
	async run({ args }) {
		const cwd = args.cwd;
		intro('Revo-Auth update');
		const paths = resolveProjectPaths(cwd);
		const manifest = readManifest(paths.manifestFile);
		if (!manifest) {
			logError('No .revo-auth/manifest.json found. Run `revo-auth init` first.');
			process.exitCode = 1;
			return;
		}
		const templatesRoot = findTemplatesRoot();

		const context = {
			serverUrl: process.env.REVO_AUTH_SERVER_URL ?? 'https://auth.revo-auth.dev',
			appId: process.env.REVO_AUTH_APP_ID ?? 'app_local_dev',
			methods: [],
			features: [],
			session: 'cookie' as const,
			preset: 'custom',
		};

		const conflicts: string[] = [];
		const updated: string[] = [];
		const unchanged: string[] = [];

		for (const [relPath, entry] of Object.entries(manifest.files)) {
			if (relPath === 'auth.config.ts') continue; // user-owned
			if (!entry.template) continue;

			const outAbs = resolve(cwd, relPath);
			const templateAbs = resolve(templatesRoot, entry.template);
			if (!existsSync(templateAbs)) continue;

			const newRendered = renderTemplate(templateAbs, context);
			const newTemplateHash = hashContent(readFileSync(templateAbs, 'utf8'));

			if (newTemplateHash === entry.templateHash && existsSync(outAbs)) {
				unchanged.push(relPath);
				continue;
			}

			if (!existsSync(outAbs)) {
				writeFileSync(outAbs, newRendered, 'utf8');
				entry.templateHash = newTemplateHash;
				entry.writtenHash = hashContent(newRendered);
				updated.push(relPath);
				continue;
			}

			const currentContent = readFileSync(outAbs, 'utf8');
			const currentHash = hashContent(currentContent);

			if (currentHash === entry.writtenHash) {
				// user hasn't touched it — overwrite silently
				writeFileSync(outAbs, newRendered, 'utf8');
				entry.templateHash = newTemplateHash;
				entry.writtenHash = hashContent(newRendered);
				updated.push(relPath);
				continue;
			}

			// 3-way merge: base is what we originally wrote (regenerate using old templateHash not possible
			// without the source — fall back to newRendered as "theirs" and previous written content as "base").
			// We approximate base by re-rendering the current template snapshot we know matches writtenHash.
			// In practice: use the stored writtenHash-matching content = currentContent's prior form.
			// Since we don't have prior content, we use an empty base heuristic: detect as conflict.
			const merge = merge3(
				// best effort: treat current template as base if no prior written known
				readFileSync(templateAbs, 'utf8'),
				currentContent,
				newRendered,
			);
			if (merge.ok) {
				writeFileSync(outAbs, merge.text, 'utf8');
				entry.templateHash = newTemplateHash;
				entry.writtenHash = hashContent(merge.text);
				updated.push(relPath);
			} else {
				writeFileSync(`${outAbs}.orig`, newRendered, 'utf8');
				writeFileSync(outAbs, merge.text, 'utf8');
				entry.templateHash = newTemplateHash;
				entry.writtenHash = hashContent(merge.text);
				conflicts.push(relPath);
			}
		}

		manifest.cliVersion = CLI_VERSION;
		writeManifest(paths.manifestFile, manifest);

		for (const f of updated) success(`updated ${f}`);
		for (const f of unchanged) info(`unchanged ${f}`);
		for (const f of conflicts) warn(`conflict in ${f} — see ${f}.orig`);

		if (conflicts.length > 0) {
			note(
				'Conflicts were written with <<<<<<< markers. Resolve them and run `revo-auth doctor`.',
				'Manual review required',
			);
		}

		if (!args.auto && args.interactive) {
			info('Interactive resolution is not yet implemented; edit conflicts manually.');
		}

		outro('Update complete.');
		// reference unused destructured to satisfy strict lint if needed
		void (manifest as Manifest);
	},
});
