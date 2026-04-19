import { existsSync, rmSync } from 'node:fs';
import { homedir } from 'node:os';
import { defineCommand } from 'citty';
import { resolve } from 'pathe';
import { info, intro, outro, success } from '../prompts.js';

export const logoutCommand = defineCommand({
	meta: { name: 'logout', description: 'Clear the admin CLI session token' },
	args: {},
	async run() {
		intro('Revo-Auth logout');
		const tokenPath = resolve(homedir(), '.revo-auth', 'token');
		if (existsSync(tokenPath)) {
			rmSync(tokenPath, { force: true });
			success(`Removed ${tokenPath}`);
		} else {
			info('Not signed in (no token file).');
		}
		outro('Done.');
	},
});
