import { defineCommand } from 'citty';
import { execa } from 'execa';
import {
	error as logError,
	info,
	intro,
	note,
	outro,
	success,
} from '../prompts.js';

async function hasDocker(): Promise<boolean> {
	try {
		await execa('docker', ['--version'], { stdio: 'ignore' });
		return true;
	} catch {
		return false;
	}
}

async function hasFly(): Promise<boolean> {
	try {
		await execa('fly', ['version'], { stdio: 'ignore' });
		return true;
	} catch {
		return false;
	}
}

export const devCommand = defineCommand({
	meta: { name: 'dev', description: 'Start a local Revo-Auth server via Docker' },
	args: {
		port: { type: 'string', default: '8787', description: 'Port to bind' },
		image: {
			type: 'string',
			default: 'ghcr.io/revo-auth/server:latest',
			description: 'Docker image tag',
		},
	},
	async run({ args }) {
		intro('Revo-Auth dev');
		const docker = await hasDocker();
		if (docker) {
			info(`Starting ${args.image} on http://localhost:${args.port}`);
			try {
				await execa(
					'docker',
					[
						'run',
						'--rm',
						'-p',
						`${args.port}:8787`,
						'-e',
						`REVO_AUTH_PORT=8787`,
						args.image,
					],
					{ stdio: 'inherit' },
				);
				success('Server exited cleanly');
				outro('Done.');
				return;
			} catch (err) {
				logError(
					`Docker run failed: ${err instanceof Error ? err.message : String(err)}`,
				);
				process.exitCode = 1;
				return;
			}
		}

		const fly = await hasFly();
		if (fly) {
			info('Docker not found; falling back to `fly dev`.');
			try {
				await execa('fly', ['dev'], { stdio: 'inherit' });
				outro('Done.');
				return;
			} catch (err) {
				logError(`fly dev failed: ${err instanceof Error ? err.message : String(err)}`);
				process.exitCode = 1;
				return;
			}
		}

		note(
			[
				'Neither Docker nor Fly CLI was found.',
				'Install Docker Desktop: https://www.docker.com/products/docker-desktop',
				'Or Fly CLI:          https://fly.io/docs/hands-on/install-flyctl/',
				`Then re-run \`revo-auth dev --port ${args.port}\`.`,
			].join('\n'),
			'Prerequisite missing',
		);
		process.exitCode = 1;
	},
});
