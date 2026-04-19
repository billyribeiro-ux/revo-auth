import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import Handlebars from 'handlebars';
import { resolve } from 'pathe';

export interface RenderContext {
	serverUrl: string;
	appId: string;
	methods: string[];
	features: string[];
	session: 'cookie' | 'bearer';
	preset: string;
	[key: string]: unknown;
}

function registerHelpers(engine: typeof Handlebars): void {
	engine.registerHelper('eq', (a: unknown, b: unknown) => a === b);
	engine.registerHelper('includes', (list: unknown, item: unknown) => {
		if (!Array.isArray(list)) return false;
		return list.includes(item);
	});
	engine.registerHelper('kebab', (input: unknown) => {
		if (typeof input !== 'string') return '';
		return input
			.replace(/([a-z0-9])([A-Z])/g, '$1-$2')
			.replace(/[_\s]+/g, '-')
			.toLowerCase();
	});
	engine.registerHelper('json', (value: unknown) => JSON.stringify(value));
	engine.registerHelper('or', (...args: unknown[]) => {
		// Handlebars passes an options arg at the end; ignore it.
		const values = args.slice(0, -1);
		return values.some(Boolean);
	});
	engine.registerHelper('and', (...args: unknown[]) => {
		const values = args.slice(0, -1);
		return values.every(Boolean);
	});
}

const engine = Handlebars.create();
registerHelpers(engine);

export function renderTemplate(
	templatePath: string,
	context: RenderContext,
): string {
	const source = readFileSync(templatePath, 'utf8');
	const compiled = engine.compile(source, { noEscape: true });
	return compiled(context);
}

export function renderTemplateFromString(
	source: string,
	context: RenderContext,
): string {
	const compiled = engine.compile(source, { noEscape: true });
	return compiled(context);
}

export function hashContent(input: string): string {
	return createHash('sha256').update(input).digest('hex');
}

/**
 * Resolve a template file path relative to the CLI package's `templates/` dir.
 * The CLI ships templates alongside the built bin; this helper produces an
 * absolute path from a logical relative path such as "sveltekit/lib/client.ts.hbs".
 */
export function resolveTemplatePath(
	templatesRoot: string,
	relPath: string,
): string {
	return resolve(templatesRoot, relPath);
}
