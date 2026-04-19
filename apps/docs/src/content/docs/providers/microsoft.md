---
title: Microsoft
description: Enable Microsoft OAuth (Entra ID) for Revo-Auth — app registration, redirect URIs, and env vars.
---

Microsoft is the critical provider for enterprise. The OAuth endpoint covers both consumer Microsoft Accounts and Entra ID (formerly Azure AD) work accounts, depending on the tenant you configure.

## 1. Register an app in Entra ID

Open the [Microsoft Entra admin center](https://entra.microsoft.com/), navigate to **Applications → App registrations → New registration**.

- **Name:** user-facing on the consent screen.
- **Supported account types:** pick **"Accounts in any organizational directory and personal Microsoft accounts"** for the broadest reach. Restrict later if you need to.
- **Redirect URI:** select **Web** and enter
  ```
  https://<your-revo-auth-server>/v1/oauth/microsoft/callback
  ```

![Microsoft Entra — App registration form](/screenshots/providers/microsoft-01-register.png)

*Screenshot placeholder: the Entra app registration page with redirect URI filled in.*

Click **Register**. Copy the **Application (client) ID**.

## 2. Create a client secret

**Certificates & secrets → Client secrets → New client secret**. Pick a duration (shorter is safer — 6 or 12 months), add a description, confirm. Copy the **Value** (not the Secret ID) immediately — Entra hides it afterwards.

## 3. Configure scopes

**API permissions → Add a permission → Microsoft Graph → Delegated permissions**. Add:

- `openid`
- `email`
- `profile`
- `User.Read`

Click **Grant admin consent** if you're in an admin seat for the tenant. Otherwise users will consent on first login.

## 4. Set env vars on the Revo-Auth server

```sh
REVO_AUTH_MICROSOFT_CLIENT_ID="..."
REVO_AUTH_MICROSOFT_CLIENT_SECRET="..."
REVO_AUTH_MICROSOFT_TENANT="common"  # or your tenant GUID
```

`common` allows any Microsoft account (work or personal). Use your tenant's GUID to restrict to your organization.

## 5. Enable the provider

```sh
pnpm dlx @revo-auth/cli add provider microsoft
```

"Sign in with Microsoft" appears on your login page.

## Tenant restrictions

If you're building a B2B SaaS and want to lock sign-ins to a single customer's Entra tenant, set `REVO_AUTH_MICROSOFT_TENANT` to that tenant's GUID. Revo-Auth will reject sign-ins from any other directory with `tenant_mismatch`.

For multi-tenant B2B, leave it at `common` and enforce tenant-level authorization in your own application after sign-in — the user's `tid` claim is available in the token, and Revo-Auth surfaces it on the user profile.

## Verifying

Click the button. Microsoft bounces you through `login.microsoftonline.com`, prompts for consent if it's the first time, and returns you signed in. Common failures:

- Redirect URI mismatch — Entra is strict; even the trailing slash matters.
- Secret expired — client secrets rotate; set a calendar reminder.
- Admin consent not granted and user can't self-consent — ask an admin.
