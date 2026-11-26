# Changelog

## Unreleased

### Added

- Add a multi-instance setting (`allow_multi_instance`, disabled by default). When enabled, launching another game skips the existing `DNF.exe` check and close confirmation. The Simplified Chinese setting is named “启用多开”.
- Save multiple accounts with passwords encrypted using Windows DPAPI. Read existing single-account credential files automatically, and allow accounts to be updated or removed individually.
- Select saved accounts and launch them together. Authenticate and launch each account sequentially, continue after individual failures, and display results per account.
- Redesign the centered login card with a translucent blue panel, account avatars, current-account indicators, rounded inputs, and a light launch button. Click an account to fill its credentials; use its checkbox to select batch launches and its context menu to remove it.
- Add Windows credential tests for legacy migration, multiple accounts, password updates, and individual account removal. Run client tests in the Windows MSVC glow CI job.

### Fixed

- Keep the launch button and navigation outside the scrolling form. Limit the saved-account list to two visible rows and show launch status in one reserved line with a separate details window.
- When accounts are checked, launch only those accounts. Editing an unchecked account no longer adds it to the batch; disabling multi-instance mode uses only the current input.
- Build 32-bit GNU hooks using a 64-bit Rust host toolchain with the i686 target.
- Install MinGW through the MSYS2 setup action and isolate 32-bit build tools from 64-bit launcher builds in CI and release workflows.
- Acknowledge the MINGW32 deprecation warning while retaining the 32-bit DLL target required by the game.

### Usage

Enable “启用多开” in Settings and save. Enter an account, enable “Remember password”, and save it; repeat for additional accounts. Check the saved accounts to launch and click “Launch selected accounts (N)”. With no checked accounts, the launch button uses the current username/password. For a checked account currently being edited, the typed password takes precedence. Disabling “Remember password” removes that account's stored credentials when saved. Right-click an account row to remove it.

This setting controls the launcher's existing-process check. Game, plugin, and server restrictions still apply. Successful compilation does not replace an in-game multi-account smoke test.
