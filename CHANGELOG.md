# Changelog

## Unreleased

### Added

- Add a multi-instance setting (`allow_multi_instance`, disabled by default). When enabled, launching another game skips the existing `DNF.exe` check and close confirmation. The Simplified Chinese setting is named “多开”.
- Save multiple accounts with passwords encrypted using Windows DPAPI. Read existing single-account credential files automatically, and allow accounts to be updated or removed individually.
- Select saved accounts and launch them together with the current account input. Authenticate and launch each account sequentially, continue after individual failures, and display results per account.
- Add Windows credential tests for legacy migration, multiple accounts, password updates, and individual account removal. Run client tests in the Windows MSVC glow CI job.

### Fixed

- Build 32-bit GNU hooks using a 64-bit Rust host toolchain with the i686 target.
- Install MinGW through the MSYS2 setup action and isolate 32-bit build tools from 64-bit launcher builds in CI and release workflows.
- Acknowledge the MINGW32 deprecation warning while retaining the 32-bit DLL target required by the game.

### Usage

Enable “多开” in Settings and save. Enter an account, enable “Remember password”, and save it; repeat for additional accounts. Select the saved accounts to launch and click “Launch selected accounts (N)”. A non-empty current username/password is included automatically, with duplicate usernames launched only once. Disabling “Remember password” removes that account's stored credentials when saved.

This setting controls the launcher's existing-process check. Game, plugin, and server restrictions still apply. Successful compilation does not replace an in-game multi-account smoke test.
