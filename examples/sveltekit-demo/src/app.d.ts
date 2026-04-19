import type { RevoAuthSession } from '@revo-auth/sdk-core';

declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			session: RevoAuthSession | null;
		}
		interface PageData {
			session: RevoAuthSession | null;
		}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
