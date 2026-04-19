export type RevoAuthErrorCode =
	| 'INVALID_CREDENTIALS'
	| 'NETWORK_ERROR'
	| 'VALIDATION_ERROR'
	| 'RATE_LIMITED'
	| 'SESSION_EXPIRED'
	| 'UNKNOWN';

export interface RevoAuthErrorJSON {
	code: RevoAuthErrorCode;
	message: string;
	requestId?: string;
}

export class RevoAuthError extends Error {
	public readonly code: RevoAuthErrorCode;
	public readonly requestId: string | undefined;
	public override readonly cause: unknown;

	constructor(
		code: RevoAuthErrorCode,
		message: string,
		options?: { requestId?: string; cause?: unknown },
	) {
		super(message);
		this.name = 'RevoAuthError';
		this.code = code;
		this.requestId = options?.requestId;
		this.cause = options?.cause;
	}

	toJSON(): RevoAuthErrorJSON {
		const out: RevoAuthErrorJSON = { code: this.code, message: this.message };
		if (this.requestId !== undefined) {
			out.requestId = this.requestId;
		}
		return out;
	}
}

export class InvalidCredentialsError extends RevoAuthError {
	constructor(
		message = 'Email or password is incorrect.',
		opts?: { requestId?: string },
	) {
		super('INVALID_CREDENTIALS', message, opts);
		this.name = 'InvalidCredentialsError';
	}
}

export class NetworkError extends RevoAuthError {
	constructor(
		message = 'Network error while contacting Revo Auth server.',
		opts?: { requestId?: string; cause?: unknown },
	) {
		super('NETWORK_ERROR', message, opts);
		this.name = 'NetworkError';
	}
}

export class ValidationError extends RevoAuthError {
	public readonly issues: ReadonlyArray<{
		path: ReadonlyArray<string | number>;
		message: string;
	}>;

	constructor(
		message: string,
		issues: ReadonlyArray<{
			path: ReadonlyArray<string | number>;
			message: string;
		}> = [],
		opts?: { requestId?: string; cause?: unknown },
	) {
		super('VALIDATION_ERROR', message, opts);
		this.name = 'ValidationError';
		this.issues = issues;
	}
}

export class RateLimitedError extends RevoAuthError {
	public readonly retryAfterSeconds: number | undefined;

	constructor(
		message = 'Rate limit exceeded.',
		opts?: { requestId?: string; retryAfterSeconds?: number },
	) {
		const base: { requestId?: string } = {};
		if (opts?.requestId !== undefined) base.requestId = opts.requestId;
		super('RATE_LIMITED', message, base);
		this.name = 'RateLimitedError';
		this.retryAfterSeconds = opts?.retryAfterSeconds;
	}
}

export class SessionExpiredError extends RevoAuthError {
	constructor(message = 'Session has expired.', opts?: { requestId?: string }) {
		super('SESSION_EXPIRED', message, opts);
		this.name = 'SessionExpiredError';
	}
}

export class UnknownError extends RevoAuthError {
	constructor(
		message = 'Unknown error.',
		opts?: { requestId?: string; cause?: unknown },
	) {
		super('UNKNOWN', message, opts);
		this.name = 'UnknownError';
	}
}

export function errorFromEnvelope(
	code: string,
	message: string,
	requestId?: string,
): RevoAuthError {
	const opts: { requestId?: string } = {};
	if (requestId !== undefined) opts.requestId = requestId;
	switch (code) {
		case 'INVALID_CREDENTIALS':
			return new InvalidCredentialsError(message, opts);
		case 'RATE_LIMITED':
			return new RateLimitedError(message, opts);
		case 'SESSION_EXPIRED':
			return new SessionExpiredError(message, opts);
		case 'VALIDATION_ERROR':
			return new ValidationError(message, [], opts);
		case 'NETWORK_ERROR':
			return new NetworkError(message, opts);
		default:
			return new RevoAuthError('UNKNOWN', message, opts);
	}
}
