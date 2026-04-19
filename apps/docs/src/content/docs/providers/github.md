---
title: GitHub
description: Enable GitHub OAuth for Revo-Auth — GitHub Developer Settings, redirect URIs, and env vars.
---

GitHub OAuth is essential for developer tooling and a sensible second provider after Google. Setup lives in your GitHub account's Developer Settings (or an organization's if you want the app owned there).

## 1. Register an OAuth app

Navigate to **Settings → Developer settings → OAuth Apps → New OAuth App** on GitHub. For an org-owned app, go through the org's settings instead.

- **Application name:** public-facing, shown on the consent screen.
- **Homepage URL:** your SvelteKit app's URL (not the Revo-Auth server).
- **Authorization callback URL:**
  ```
  https://<your-revo-auth-server>/v1/oauth/github/callback
  ```

![GitHub — OAuth App registration form](/screenshots/providers/github-01-register.png)

*Screenshot placeholder: the GitHub OAuth app registration page.*

Click **Register application**. GitHub shows the **Client ID**. Click **Generate a new client secret** — copy the secret immediately, GitHub only shows it once.

## 2. Set env vars on the Revo-Auth server

```sh
REVO_AUTH_GITHUB_CLIENT_ID="Iv1.abc..."
REVO_AUTH_GITHUB_CLIENT_SECRET="..."
```

For Fly.io: `fly secrets set REVO_AUTH_GITHUB_CLIENT_ID=... REVO_AUTH_GITHUB_CLIENT_SECRET=...`.

## 3. Enable the provider

```sh
pnpm dlx @revo-auth/cli add provider github
```

The "Sign in with GitHub" button appears on your login page.

## Scopes

By default Revo-Auth requests `read:user` and `user:email`. That gives the server enough to populate the `users` row. If you need more (e.g. `repo` for a dev tool that reads repos), add scopes in the dashboard under **Providers → GitHub → Scopes**. Users will see the expanded consent screen on next login.

## Email privacy

GitHub users can set their primary email to private. When that happens, the `email` field on the OAuth response is null. Revo-Auth falls back to the user's verified emails via the `/user/emails` API (which requires `user:email`) and picks the primary verified address. If none exists, the login fails with `email_required` — your UI should explain.

## Verifying

Click the button. You should see GitHub's authorization screen, grant access, and return signed in. Common issues:

- Callback URL typo — no trailing slash, scheme matters.
- Client secret regenerated on GitHub but not updated in Revo-Auth — rotate in both places.
- Org-owned app but the user isn't a member — GitHub blocks with a clear message.

Run `pnpm dlx @revo-auth/cli doctor` to sanity-check.
