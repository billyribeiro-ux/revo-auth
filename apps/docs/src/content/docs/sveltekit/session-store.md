---
title: Session store
description: A runes-based client store that keeps the signed-in user reactive across your UI.
---

`@revo-auth/sdk-sveltekit/client` ships a runes-based session store so client components can read `user`, `session`, and `isAuthenticated` reactively without prop-drilling or manual subscriptions.

## Wiring the store

In your root layout:

```svelte
<!-- src/routes/+layout.svelte -->
<script lang="ts">
  import { setSessionContext } from "@revo-auth/sdk-sveltekit/client";

  let { data, children } = $props();
  setSessionContext(() => data.user);
</script>

{@render children()}
```

And in `+layout.server.ts`:

```ts
export const load = async (event) => {
  return { user: event.locals.user };
};
```

`setSessionContext` stores a reactive getter on Svelte's context. The getter is called by `useSession()` in any descendant component.

## Reading the session

```svelte
<script lang="ts">
  import { useSession } from "@revo-auth/sdk-sveltekit/client";

  const session = useSession();
</script>

{#if session.isAuthenticated}
  <p>Hi {session.user.name}</p>
{:else}
  <a href="/login">Sign in</a>
{/if}
```

`session` is a proxy over `$derived` values. Accessing `session.user`, `session.session`, or `session.isAuthenticated` subscribes your component to changes — nothing else required.

## Updates without a reload

After an action that mutates the session (login, logout, passkey registration, step-up), the server sets or clears the cookie and the returned redirect re-fetches `+layout.server.ts`. SvelteKit re-runs the layout load, `data.user` changes, and every component using `useSession()` rerenders. No manual invalidation, no websockets, no polling.

For client-only transitions (e.g. an in-place logout button), call `invalidateAll()` from `$app/navigation` after the SDK call — SvelteKit will re-run loads and the store will pick up the change.

## Server-side parity

Server loads and endpoints should read `event.locals.user` directly. The store exists for client-reactive UI only — never call `useSession()` from a `.server.ts` file. The types prevent it but the pattern is worth stating.

## Logging out

```svelte
<script>
  import { goto, invalidateAll } from "$app/navigation";
  import { sdk } from "$lib/auth";

  async function signOut() {
    await sdk.session.revoke();
    await invalidateAll();
    await goto("/");
  }
</script>

<button onclick={signOut}>Sign out</button>
```

`sdk.session.revoke()` tells the server to invalidate the session and clears the cookie via a `Set-Cookie` header on the response. `invalidateAll` re-runs the layout load, which sees `locals.user === null`, and the UI updates. `goto("/")` is cosmetic — you can drop it if you want the user to stay on the current page.
