# Claude Usage

Shows how much of your Claude plan you have used, right in the Mac menu bar.

```
9% (2h) · 68% · 100%
```

From left to right:

1. Your current 5-hour session, and the time left until it resets
2. Your weekly limit
3. Your weekly Fable limit, if your plan has one

It updates every 2 minutes.

## What you need

- A Mac
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code), logged in with your Claude account (Pro, Max or Team)

## Install

1. Download the `.dmg` from [Releases](https://github.com/mklinovsky/claude-usage/releases).
2. Drag **Claude Usage** into **Applications**.
3. Open it. macOS will block it the first time because the app is not from the App Store. Go to **System Settings → Privacy & Security** and click **Open Anyway**.
4. If your Mac asks to access `Claude Code-credentials`, click **Always Allow**.

To start it automatically, add it in **System Settings → General → Login Items**.

## Good to know

- It uses your Claude Code login. If you see `⚠`, hover over it to see what went wrong. If the token expired, run `claude` in a terminal once.
- Your login stays on your Mac and is only sent to Anthropic.
- This is not an official Anthropic app. It may stop working if Anthropic changes things.

## Build it yourself

You need [Rust](https://rustup.rs), Node.js and [pnpm](https://pnpm.io).

```sh
pnpm install
pnpm tauri build
```
