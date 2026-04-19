import * as clack from '@clack/prompts';
import type { MultiSelectOptions, SelectOptions } from '@clack/prompts';
import pc from 'picocolors';

interface PromptOption<T> {
	value: T;
	label: string;
	hint?: string | undefined;
}

function toClackOptions<T>(options: PromptOption<T>[]): { value: T; label: string; hint?: string }[] {
	return options.map((o) => {
		const base: { value: T; label: string; hint?: string } = {
			value: o.value,
			label: o.label,
		};
		if (typeof o.hint === 'string') base.hint = o.hint;
		return base;
	});
}

export function intro(title: string): void {
	clack.intro(pc.bgMagenta(pc.black(` ${title} `)));
}

export function outro(message: string): void {
	clack.outro(message);
}

export function note(body: string, title?: string): void {
	clack.note(body, title);
}

export function success(message: string): void {
	clack.log.success(pc.green(message));
}

export function info(message: string): void {
	clack.log.info(message);
}

export function warn(message: string): void {
	clack.log.warn(pc.yellow(message));
}

export function error(message: string): void {
	clack.log.error(pc.red(message));
}

function ensureNotCancelled<T>(value: T | symbol): T {
	if (clack.isCancel(value)) {
		clack.cancel('Cancelled.');
		process.exit(0);
	}
	return value as T;
}

export async function askText(
	message: string,
	defaultValue?: string,
	placeholder?: string,
): Promise<string> {
	const result = await clack.text({
		message,
		defaultValue: defaultValue ?? '',
		placeholder: placeholder ?? defaultValue ?? '',
	});
	return ensureNotCancelled(result);
}

export async function askSelect<T extends string>(
	message: string,
	options: PromptOption<T>[],
	initialValue?: T,
): Promise<T> {
	const clackOptions = toClackOptions(options) as SelectOptions<T>['options'];
	const opts: SelectOptions<T> = initialValue !== undefined
		? { message, options: clackOptions, initialValue }
		: { message, options: clackOptions };
	const result = await clack.select<T>(opts);
	return ensureNotCancelled(result);
}

export async function askMultiSelect<T extends string>(
	message: string,
	options: PromptOption<T>[],
	initialValues?: T[],
): Promise<T[]> {
	const clackOptions = toClackOptions(options) as MultiSelectOptions<T>['options'];
	const opts: MultiSelectOptions<T> = {
		message,
		options: clackOptions,
		initialValues: initialValues ?? [],
		required: false,
	};
	const result = await clack.multiselect<T>(opts);
	return ensureNotCancelled(result);
}

export async function askConfirm(message: string, initial = true): Promise<boolean> {
	const result = await clack.confirm({ message, initialValue: initial });
	return ensureNotCancelled(result);
}

export function spinner(): { start: (msg: string) => void; stop: (msg?: string) => void } {
	return clack.spinner();
}
