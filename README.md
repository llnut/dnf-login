# dnf-login

A launcher for Dungeon & Fighter (DNF) written in Rust.

![Screenshot](https://github.com/user-attachments/assets/40e5dad7-a607-4765-aec4-e2b4574b4a29)

> [!WARNING]
> This launcher does not guarantee compatibility with other launchers. Accounts
> registered elsewhere may not work here, and vice versa — be prepared for the
> possibility that existing accounts become inaccessible after switching.
>
> The API is not yet stable and may change between versions. Not recommended for
> production use.

## Features

### Server

- Account login, registration, password change, and password recovery
- Input validation on all fields before any database access
- Per-IP rate limiting (10 requests per 60-second window)
- Encrypted communication: all request and response bodies are AES-256-GCM ciphertext
- Game server IP endpoint: the client fetches the game server IP from the server at login, no manual configuration needed

### Client

- Multi-language support: English, Simplified Chinese, Traditional Chinese, Japanese, Korean
- Optional credential storage encrypted with Windows DPAPI
- Custom wallpaper directory with five fill modes: Tile, Stretch, Fill, Center, Fit
- Detects whether the game is already running before launching

### ijl15.dll

- Proxy DLL that replaces the original `ijl15.dll` loaded by DNF.exe
- DNS redirect: hooks `gethostbyname` to redirect "start.dnf.tw" to the game server IP
- Plugin loader: scans the plugin directory for `.dll` files and loads them
- Works standalone with `GameServerIP.toml`, or with the client via environment variables

| Mode | IP source | Plugin path |
|---|---|---|
| With client | `GAME_SERVER_IP` env var, set automatically from server response | `DNF_PLUGIN_PATH` env var, set from `Config.toml` |
| Standalone | `GameServerIP.toml` in game directory | Default `plugins` directory |


## Building

**Prerequisites:**

- [Rust](https://rustup.rs/) toolchain (stable channel)

**Build commands:**

```bash
# Build all crates
cargo build --release

# Build a specific crate
cargo build --release -p dnf-server
cargo build --release -p dnf-client

# Build ijl15-hook, requires 32-bit target
rustup target add i686-pc-windows-gnu
cargo build --release --target i686-pc-windows-gnu -p ijl15-hook

# Run tests
cargo test
```

The client crate is Windows-only. Cross-compiling from Linux:

```bash
cargo build --release -p dnf-client --target x86_64-pc-windows-gnu
```


## Server Deployment

**Requirements:**

- Linux (x86_64, aarch64) or Windows (x86_64)
- MySQL 5.x with an existing DNF database (`d_taiwan` and related schemas)
- RSA private key matching the game's public key

**Setup:**

1. Download the archive for your platform from the [Releases](https://github.com/llnut/dnf-login/releases) page and extract it.
2. Copy `.env.example` (included in the archive) to `.env` in the same directory and fill in the required values.
3. Place the RSA private key at `/data/privatekey.pem`, or set `RSA_PRIVATE_KEY_PATH` to an alternative path.
4. Run the server:

```bash
./dnf-gate-server
```

**Configuration variables:**

| Variable | Required | Default | Notes |
|---|---|---|---|
| `AES_KEY` | yes | | 64 hex characters (32 bytes). Generate with `openssl rand -hex 32`. Must match the client. |
| `DB_PASSWORD` | yes | | Plain text. Special characters do not need escaping. |
| `DB_HOST` | no | `127.0.0.1` | |
| `DB_PORT` | no | `3306` | |
| `DB_USER` | no | `game` | |
| `DB_NAME` | no | `d_taiwan` | |
| `RSA_PRIVATE_KEY_PATH` | no | `/data/privatekey.pem` | |
| `BIND_ADDRESS` | no | `0.0.0.0:5505` | HTTP listener address. |
| `INITIAL_CERA` | no | `1000` | |
| `INITIAL_CERA_POINT` | no | `0` | |
| `GAME_SERVER_IP` | no | | Reported to clients via the GET endpoint. Clients pass it to ijl15.dll as an environment variable. |
| `TLS_CERT_PATH` | no | | PEM certificate file (may include intermediate CA chain). TLS is enabled only when both `TLS_CERT_PATH` and `TLS_KEY_PATH` are set. |
| `TLS_KEY_PATH` | no | | PEM private key file. |
| `TLS_BIND_ADDRESS` | no | `0.0.0.0:5504` | HTTPS listener address. |
| `TLS_ONLY` | no | `false` | When `true`, the HTTP listener is disabled. Requires TLS to be configured. |
| `RUST_LOG` | no | `info` | e.g. `info,dnf_gate_server=debug` |


## Client Deployment

1. Download the archive for your platform from the [Releases](https://github.com/llnut/dnf-login/releases) page and extract the executable into the game directory alongside `DNF.exe`.

   Two rendering backends are provided per target:

   | Asset | Backend | Note |
   |---|---|---|
   | `dnf-launcher-wgpu-*` | DX12 or Vulkan | Needs DX12 or Vulkan driver support. |
   | `dnf-launcher-glow-*` | OpenGL 2.0+ | Smaller binary and lower memory footprint. Needs an OpenGL 2.0+ driver. |

2. Copy `Config.example.toml` to `Config.toml` in the same directory and set `server_url` and `aes_key`. These can also be configured from the in-app settings screen.

```toml
server_url   = "http://192.168.200.131:5505"
aes_key      = "<64 hex characters matching the server>"
plugins_path  = "plugins"
```


### ijl15.dll Setup

Place `ijl15.dll` from the release archive in the game directory alongside `DNF.exe`. When used with the client, no additional configuration is needed. When used standalone, create `GameServerIP.toml` in the game directory:

```toml
game_server_ip = "192.168.200.131"
```

### Plugin Loading

Place 32-bit DLL files in the `plugins` directory alongside `DNF.exe`. The path can be changed via `plugins_path` in `Config.toml`. All DLLs are loaded by `ijl15.dll` during game startup.

To disable plugin loading: uncheck the toggle in the settings screen, set `plugin_inject_enabled = false` in `Config.toml`, or set `DNF_PLUGIN_ENABLED=0` as an environment variable.


## License

MIT License - see the [LICENSE](LICENSE) file for details.
