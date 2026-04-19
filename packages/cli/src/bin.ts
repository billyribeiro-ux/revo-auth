import { defineCommand, runMain } from 'citty';
import pc from 'picocolors';
import { addCommand } from './commands/add.js';
import { devCommand } from './commands/dev.js';
import { doctorCommand } from './commands/doctor.js';
import { initCommand } from './commands/init.js';
import { logoutCommand } from './commands/logout.js';
import { uiCommand } from './commands/ui.js';
import { updateCommand } from './commands/update.js';
import { detectPackageManager } from './detect/pm.js';
import { error as logError } from './prompts.js';

const PKG_VERSION = '0.1.0';

const main = defineCommand({
	meta: {
		name: 'revo-auth',
		version: PKG_VERSION,
		description: 'Revo-Auth CLI — drop-in auth for SvelteKit',
	},
	subCommands: {
		init: initCommand,
		add: addCommand,
		update: updateCommand,
		ui: uiCommand,
		dev: devCommand,
		doctor: doctorCommand,
		logout: logoutCommand,
	},
	setup() {
		// Enforce pnpm on every command except `logout` (which operates outside a project).
		const invokedSub = process.argv[2];
		if (invokedSub && invokedSub !== 'logout' && !invokedSub.startsWith('-')) {
			const check = detectPackageManager(process.cwd());
			if (!check.ok) {
				logError(check.message);
				process.exit(1);
			}
		}
	},
});

runMain(main).catch((err: unknown) => {
	const message = err instanceof Error ? err.message : String(err);
	// eslint-disable-next-line no-console
	console.error(pc.red(`revo-auth: ${message}`));
	process.exit(1);
});
