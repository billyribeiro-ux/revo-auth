import type { z } from 'zod';

export interface CreateFormOptions<T extends Record<string, unknown>> {
	schema: z.ZodType<T>;
	initial: T;
	onSubmit: (values: T) => Promise<void> | void;
}

export interface FormController<T extends Record<string, unknown>> {
	values: T;
	errors: Partial<Record<keyof T, string>>;
	submitting: boolean;
	submit: () => Promise<void>;
	setError: (key: keyof T, message: string | undefined) => void;
	reset: () => void;
	attach: (form: HTMLFormElement) => () => void;
}

/**
 * Runes-based form controller. The returned object exposes reactive
 * `values`, `errors`, and `submitting` properties, plus an `attach`
 * method intended for use with Svelte 5's `{@attach}` directive on a
 * `<form>` element. It prevents default submission, validates against
 * the given zod schema, and invokes `onSubmit` on success.
 */
export function createForm<T extends Record<string, unknown>>(
	options: CreateFormOptions<T>
): FormController<T> {
	const { schema, initial, onSubmit } = options;

	const values = $state<T>({ ...initial });
	const errors = $state<Partial<Record<keyof T, string>>>({});
	let submitting = $state(false);

	function clearErrors(): void {
		for (const key of Object.keys(errors) as Array<keyof T>) {
			delete errors[key];
		}
	}

	function setError(key: keyof T, message: string | undefined): void {
		if (message === undefined) {
			delete errors[key];
		} else {
			errors[key] = message;
		}
	}

	function reset(): void {
		for (const key of Object.keys(values) as Array<keyof T>) {
			(values as Record<keyof T, unknown>)[key] = (
				initial as Record<keyof T, unknown>
			)[key];
		}
		clearErrors();
		submitting = false;
	}

	async function submit(): Promise<void> {
		if (submitting) return;
		clearErrors();
		const result = schema.safeParse(values);
		if (!result.success) {
			for (const issue of result.error.issues) {
				const key = issue.path[0];
				if (typeof key === 'string' || typeof key === 'number') {
					const k = key as keyof T;
					if (errors[k] === undefined) errors[k] = issue.message;
				}
			}
			return;
		}
		submitting = true;
		try {
			await onSubmit(result.data);
		} finally {
			submitting = false;
		}
	}

	function attach(form: HTMLFormElement): () => void {
		const handler = (event: SubmitEvent): void => {
			event.preventDefault();
			void submit();
		};
		form.addEventListener('submit', handler);
		return () => {
			form.removeEventListener('submit', handler);
		};
	}

	return {
		get values() {
			return values;
		},
		get errors() {
			return errors;
		},
		get submitting() {
			return submitting;
		},
		submit,
		setError,
		reset,
		attach
	};
}
