import { existsSync } from 'node:fs';
import { defineCommand } from 'citty';
import { mergeAuthConfig } from '../ast/merge-config.js';
import { resolveProjectPaths } from '../detect/paths.js';
import { error as logError, info, intro, outro, success, warn } from '../prompts.js';
import { findTemplatesRoot, scaffoldFiles, type ScaffoldFile } from '../scaffold.js';
import { emptyManifest, readManifest, writeManifest } from '../templates/manifest.js';

const CLI_VERSION = '0.1.0';

const FEATURES = [
	'email',
	'google',
	'github',
	'microsoft',
	'apple',
	'discord',
	'passkeys',
	'totp',
	'magic-link',
	'organizations',
	'audit-log',
	'custom-fields',
	'anonymous-mode',
] as const;

type Feature = (typeof FEATURES)[number];

export const addCommand = defineCommand({
	meta: { name: 'add', description: 'Add a feature (provider or module) to an existing project' },
	args: {
		feature: { type: 'positional', required: true, description: `One of: ${FEATURES.join(', ')}` },
		force: { type: 'boolean', description: 'Overwrite existing files', default: false },
		cwd: { type: 'string', description: 'Project root', default: process.cwd() },
	},
	async run({ args }) {
		const cwd = args.cwd;
		const feature = args.feature as Feature;
		if (!FEATURES.includes(feature)) {
			logError(`Unknown feature "${feature}". Available: ${FEATURES.join(', ')}`);
			process.exitCode = 1;
			return;
		}
		intro(`Revo-Auth add ${feature}`);

		const paths = resolveProjectPaths(cwd);
		const manifest = readManifest(paths.manifestFile) ?? emptyManifest(CLI_VERSION);
		const templatesRoot = findTemplatesRoot();

		const files: ScaffoldFile[] = [];
		let configPatch: Record<string, unknown> = {};

		switch (feature) {
			case 'google':
			case 'github':
			case 'microsoft':
			case 'apple': {
				const template = `sveltekit/providers/${feature}.ts.hbs`;
				files.push({
					template,
					outRel: `src/lib/auth/providers/${feature}.ts`,
					...(args.force ? {} : { skipIfUnmanagedExists: true }),
				});
				configPatch = { methods: { [feature]: { enabled: feature !== 'apple' } } };
				break;
			}
			case 'discord': {
				// no template — just config patch with manual setup instructions
				configPatch = { methods: { discord: { enabled: true } } };
				break;
			}
			case 'email':
			case 'passkeys':
			case 'totp':
			case 'magic-link': {
				const key =
					feature === 'magic-link'
						? 'magicLink'
						: feature === 'passkeys'
							? 'passkeys'
							: feature;
				configPatch = { methods: { [key]: { enabled: true } } };
				break;
			}
			case 'organizations': {
				configPatch = { features: { organizations: { enabled: true } } };
				break;
			}
			case 'audit-log': {
				configPatch = { features: { auditLog: { enabled: true } } };
				break;
			}
			case 'custom-fields': {
				configPatch = { features: { customFields: { enabled: true } } };
				break;
			}
			case 'anonymous-mode': {
				configPatch = { features: { anonymousMode: { enabled: true } } };
				break;
			}
		}

		if (files.length > 0) {
			const context = {
				serverUrl: process.env.REVO_AUTH_SERVER_URL ?? 'https://auth.revo-auth.dev',
				appId: process.env.REVO_AUTH_APP_ID ?? 'app_local_dev',
				methods: [feature],
				features: [],
				session: 'cookie' as const,
				preset: 'custom',
			};
			const result = scaffoldFiles(cwd, templatesRoot, files, context, manifest);
			for (const w of result.written) success(`wrote ${w}`);
			for (const s of result.skipped) warn(`skipped ${s} (exists; use --force)`);
		}

		if (!existsSync(paths.authConfig)) {
			warn('auth.config.ts not found — run `revo-auth init` first.');
		} else {
			const merge = mergeAuthConfig(paths.authConfig, configPatch);
			if (merge.added.length > 0) success(`auth.config.ts: added ${merge.added.join(', ')}`);
			else info('auth.config.ts already has this feature');
			for (const c of merge.conflicts) logError(c);
		}

		writeManifest(paths.manifestFile, manifest);
		outro(`Added ${feature}.`);
	},
});
