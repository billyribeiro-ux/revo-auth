---
title: revo-auth ui
description: Copy UI primitives from @revo-auth/ui-sveltekit into your project for full customization.
---

`revo-auth ui` pulls component source from `@revo-auth/ui-sveltekit` directly into your project. This is the shadcn pattern — you own the code, you can restyle it, you can delete parts you don't need, and you never have to fight a library author's design decisions.

## Listing available components

```sh
pnpm dlx @revo-auth/cli ui list
```

Current components: `LoginForm`, `SignupForm`, `OAuthButtons`, `PasskeyButton`, `TotpEnroll`, `TotpPrompt`, `RecoveryCodesDownload`, `StepUpPrompt`, `UserMenu`, `SessionIndicator`.

## Adding a component

```sh
pnpm dlx @revo-auth/cli ui add LoginForm
```

Writes `src/lib/components/revo-auth/LoginForm.svelte` (and any co-located helpers). The component imports from `@revo-auth/sdk-sveltekit` for transport and uses your project's `tokens.css` for styling — no additional install needed.

Repeat for each component you want. The `revo-auth.json` `uiComponents` array tracks what's been added.

## Customizing

Edit the file in place. It's yours. `update` won't touch it. If you want to re-pull the upstream version, `ui add --force LoginForm` shows a diff against your edited copy and overwrites only after you confirm.

## Styling

All components consume CSS custom properties from the tokens file copied in by `init` (`src/styles/tokens.css` or your project's equivalent). Override the tokens to restyle globally:

```css
:root {
  --accent-base: oklch(70% 0.2 30); /* warmer brand orange */
}
```

Components won't change, but every button, focus ring, and highlight will track the new token. That's the whole point.

## Removing

```sh
pnpm dlx @revo-auth/cli ui remove LoginForm
```

Deletes the file and removes the entry from `revo-auth.json`. It does not search your codebase for import references — if you still import the component somewhere, you'll get a TypeScript error on next build. That's the desired failure mode.
