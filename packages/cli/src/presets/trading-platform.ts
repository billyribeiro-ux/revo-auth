import type { Preset } from './types.js';

const preset: Preset = {
	name: 'trading-platform',
	methods: ['email', 'google', 'totp'],
	session: 'cookie',
	features: ['organizations', 'audit-log', 'custom-fields'],
};

export default preset;
