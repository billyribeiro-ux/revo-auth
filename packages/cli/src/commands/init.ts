import { existsSync, mkdirSync } from 'node:fs';
import { defineCommand } from 'citty';
import { execa } from 'execa';
import { ofetch } from 'ofetch';
import pc from 'picocolors';
import { mergeAuthConfig } from '../ast/merge-config.js';
import { mergeEnvFile } from '../ast/merge-env.js';
import { mergeHooksServer } from '../ast/merge-hooks.js';
import { type InitConfig, initConfigSchema } from '../config.js';
import { detectFramework } from '../detect/framework.js';
import { resolveProjectPaths } from '../detect/paths.js';
import { detectPackageManager } from '../detect/pm.js';
import { getPreset, listPresets } from '../presets/index.js';
import {
	askConfirm,
	askMultiSelect,
	askSelect,
	askText,
	info,
	intro,
	error as logError,
	note,
	outro,
	spinner,
	success,
	warn,
} from '../prompts.js';
import {
	type ScaffoldFile,
	appendGitignore,
	findTemplatesRoot,
	scaffoldFiles,
} from '../scaffold.js';
import {
	emptyManifest,
	readManifest,
	writeManifest,
} from '../templates/manifest.js';

const CLI_VERSION = '0.1.0';

export const initCommand = defineCommand({
	meta: {
		name: 'init',
		description: 'Initialize Revo-Auth in a SvelteKit project',
	},
	args: {
		preset: { type: 'string', description: 'Preset name', required: false },
		server: { type: 'string', description: 'Auth server URL', required: false },
		yes: {
			type: 'boolean',
			description: 'Accept defaults without prompts',
			default: false,
		},
		cwd: {
			type: 'string',
			description: 'Project root',
			default: process.cwd(),
		},
	},
	async run({ args }) {
		const cwd = args.cwd;
		intro('Revo-Auth init');

		const pm = detectPackageManager(cwd);
		if (!pm.ok) {
			logError(pm.message);
			process.exitCode = 1;
			return;
		}

		const framework = detectFramework(cwd);
		if (!framework.ok) {
			for (const err of framework.errors) logError(err);
			if (!framework.info.hasTypeScript) {
				const fix = args.yes
					? true
					: await askConfirm('Install TypeScript now?', true);
				if (fix) {
					const s = spinner();
					s.start('Installing typescript');
					await execa('pnpm', ['add', '-D', 'typescript'], {
						cwd,
						stdio: 'pipe',
					});
					s.stop('typescript installed');
					if (!framework.info.hasTsConfig) {
						await execa(
							'node',
							[
								'-e',
								"require('fs').writeFileSync('tsconfig.json', JSON.stringify({ compilerOptions: { target: 'ES2022', module: 'ESNext', moduleResolution: 'bundler', strict: true, esModuleInterop: true, skipLibCheck: true }, include: ['src/**/*.ts'] }, null, 2))",
							],
							{ cwd },
						);
					}
				} else {
					process.exitCode = 1;
					return;
				}
			}
			if (
				framework.info.sveltekitMajor !== null &&
				framework.info.sveltekitMajor < 2
			) {
				process.exitCode = 1;
				return;
			}
			if (framework.info.framework !== 'sveltekit') {
				process.exitCode = 1;
				return;
			}
		}

		const config = await gatherConfig(args);
		const paths = resolveProjectPaths(cwd);

		if (!existsSync(paths.manifestDir))
			mkdirSync(paths.manifestDir, { recursive: true });
		const manifest =
			readManifest(paths.manifestFile) ?? emptyManifest(CLI_VERSION);

		// Register app if master key present
		const masterKey = process.env.REVO_AUTH_MASTER_KEY;
		let appId = config.appId;
		let publicKey = 'replace-me-public';
		let secretKey = 'replace-me-secret';
		if (masterKey) {
			const s = spinner();
			s.start('Registering app with server');
			try {
				const response = await ofetch<{
					appId: string;
					publicKey: string;
					secretKey: string;
				}>('/admin/apps', {
					baseURL: config.serverUrl,
					method: 'POST',
					headers: { authorization: `Bearer ${masterKey}` },
					body: { name: framework.info.packageJsonPath, preset: config.preset },
				});
				appId = response.appId;
				publicKey = response.publicKey;
				secretKey = response.secretKey;
				s.stop(pc.green(`Registered app ${appId}`));
			} catch (err) {
				s.stop(pc.yellow('Could not register app automatically'));
				warn(
					`Server registration failed (${err instanceof Error ? err.message : String(err)}). Writing placeholders.`,
				);
			}
		} else {
			info('No REVO_AUTH_MASTER_KEY found — writing .env placeholders.');
		}

		const templatesRoot = findTemplatesRoot();
		const files: ScaffoldFile[] = [
			{ template: 'sveltekit/auth.config.ts.hbs', outRel: 'auth.config.ts' },
			{
				template: 'sveltekit/lib/client.ts.hbs',
				outRel: 'src/lib/auth/client.ts',
			},
			{
				template: 'sveltekit/lib/server.ts.hbs',
				outRel: 'src/lib/auth/server.ts',
			},
			{
				template: 'sveltekit/lib/middleware.ts.hbs',
				outRel: 'src/lib/auth/middleware.ts',
			},
			{
				template: 'sveltekit/lib/types.ts.hbs',
				outRel: 'src/lib/auth/types.ts',
			},
			{
				template: 'sveltekit/routes/login.svelte.hbs',
				outRel: 'src/routes/auth/login/+page.svelte',
			},
			{
				template: 'sveltekit/routes/signup.svelte.hbs',
				outRel: 'src/routes/auth/signup/+page.svelte',
			},
			{
				template: 'sveltekit/routes/callback.server.ts.hbs',
				outRel: 'src/routes/auth/callback/+server.ts',
			},
			{
				template: 'sveltekit/routes/verify-email.svelte.hbs',
				outRel: 'src/routes/auth/verify-email/+page.svelte',
			},
			{
				template: 'sveltekit/routes/reset-password.svelte.hbs',
				outRel: 'src/routes/auth/reset-password/+page.svelte',
			},
		];

		for (const method of config.methods) {
			const providerTemplate = `sveltekit/providers/${method}.ts.hbs`;
			if (existsSync(`${templatesRoot}/${providerTemplate}`)) {
				files.push({
					template: providerTemplate,
					outRel: `src/lib/auth/providers/${method}.ts`,
				});
			}
		}

		const context = {
			serverUrl: config.serverUrl,
			appId,
			methods: config.methods,
			features: config.features,
			session: config.session,
			preset: config.preset,
		};

		const scaffoldResult = scaffoldFiles(
			cwd,
			templatesRoot,
			files,
			context,
			manifest,
		);
		for (const w of scaffoldResult.written) success(`wrote ${w}`);
		for (const s of scaffoldResult.skipped)
			warn(`skipped ${s} (already exists, unmanaged)`);

		// auth.config.ts is user-owned — ensure defaults merge (no overwrite) on re-init.
		if (existsSync(paths.authConfig) && manifest.files['auth.config.ts']) {
			// freshly scaffolded; already written by scaffoldFiles
		} else {
			mergeAuthConfig(paths.authConfig, {
				serverUrl: config.serverUrl,
				appId,
				session: { strategy: config.session },
			});
		}

		// hooks.server.ts merge
		const hooks = mergeHooksServer(paths.hooksServer);
		if (hooks.created) success('created src/hooks.server.ts');
		else if (hooks.changed)
			success('updated src/hooks.server.ts (wrapped existing handle)');
		else info('src/hooks.server.ts already wired');

		// .env
		const envEntries = [
			{ key: 'REVO_AUTH_SERVER_URL', value: config.serverUrl },
			{ key: 'REVO_AUTH_APP_ID', value: appId },
			{ key: 'REVO_AUTH_PUBLIC_KEY', value: publicKey },
			{ key: 'REVO_AUTH_SECRET_KEY', value: secretKey },
		];
		const envResult = mergeEnvFile(paths.envFile, envEntries);
		const envExampleResult = mergeEnvFile(
			paths.envExample,
			envEntries.map((e) => ({
				key: e.key,
				value: e.key === 'REVO_AUTH_SERVER_URL' ? config.serverUrl : '',
			})),
		);
		if (envResult.added.length > 0)
			success(`.env: added ${envResult.added.join(', ')}`);
		if (envExampleResult.added.length > 0)
			success(`.env.example: added ${envExampleResult.added.join(', ')}`);

		// .gitignore
		const added = appendGitignore(paths.gitignore, [
			'.revo-auth/',
			'auth.secret.*',
			'.env',
		]);
		if (added.length > 0) success(`.gitignore: added ${added.join(', ')}`);

		// Install deps
		if (!args.yes || process.env.REVO_AUTH_SKIP_INSTALL !== '1') {
			const doInstall = args.yes
				? true
				: await askConfirm('Install SDK + UI packages now?', true);
			if (doInstall && process.env.REVO_AUTH_SKIP_INSTALL !== '1') {
				const s = spinner();
				s.start(
					'pnpm install @revo-auth/sdk-sveltekit @revo-auth/ui-sveltekit',
				);
				try {
					await execa(
						'pnpm',
						['install', '@revo-auth/sdk-sveltekit', '@revo-auth/ui-sveltekit'],
						{ cwd, stdio: 'pipe' },
					);
					s.stop('Dependencies installed');
				} catch (err) {
					s.stop(pc.yellow('Install failed; you can run it manually'));
					warn(err instanceof Error ? err.message : String(err));
				}
			}
		}

		writeManifest(paths.manifestFile, manifest);
		success('wrote .revo-auth/manifest.json');

		note(
			[
				`Server: ${pc.cyan(config.serverUrl)}`,
				`Preset: ${pc.cyan(config.preset)}`,
				`Methods: ${config.methods.join(', ')}`,
				'',
				'Next steps:',
				'  1. Set OAuth credentials in .env (see provider setup docs).',
				'  2. pnpm dev',
				'  3. Visit /auth/login to verify the flow.',
			].join('\n'),
			'Revo-Auth ready',
		);
		outro(pc.green('Done.'));
	},
});

async function gatherConfig(args: {
	preset?: string | undefined;
	server?: string | undefined;
	yes: boolean;
}): Promise<InitConfig> {
	const defaultServer = 'https://auth.revo-auth.dev';
	let presetName = args.preset ?? 'trading-platform';
	let serverUrl = args.server ?? defaultServer;

	if (!args.yes && args.preset === undefined) {
		presetName = await askSelect(
			'Pick a preset',
			listPresets(),
			'trading-platform',
		);
	}
	if (!args.yes && args.server === undefined) {
		serverUrl = await askText('Auth server URL', defaultServer);
	}
	const preset = getPreset(presetName);
	if (!preset) {
		throw new Error(`Unknown preset "${presetName}"`);
	}

	let methods = preset.methods;
	let session = preset.session;
	if (!args.yes) {
		methods = await askMultiSelect(
			'Methods',
			[
				{ value: 'email', label: 'Email + password' },
				{ value: 'google', label: 'Google OAuth' },
				{ value: 'github', label: 'GitHub OAuth' },
				{ value: 'microsoft', label: 'Microsoft OAuth' },
				{ value: 'passkeys', label: 'Passkeys (WebAuthn)' },
				{ value: 'totp', label: 'TOTP 2FA' },
				{ value: 'magic-link', label: 'Magic link' },
			],
			preset.methods,
		);
		session = await askSelect(
			'Session strategy',
			[
				{ value: 'cookie', label: 'Cookie (recommended)' },
				{ value: 'bearer', label: 'Bearer token' },
			],
			preset.session,
		);
	}

	return initConfigSchema.parse({
		serverUrl,
		preset: presetName,
		methods,
		features: preset.features,
		session,
		appId: `app_${presetName.replace(/-/g, '_')}`,
	});
}
