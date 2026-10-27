# Architecture

> Spanish version: [ARCHITECTURE.md](ARCHITECTURE.md). The Spanish version is authoritative. In case of conflict, the Spanish version prevails.

## Overview

Cardumen Deck is a Tauri 2 application with a Vue 3 interface and a Rust backend.

```text
Vue 3 interface
  -> Tauri commands (invoke)
    -> Rust modules
      -> local processes, files, Git, logs, and metrics
```

There is no remote backend. The application reads and writes local files.

## Interface

- `src/App.vue`: main composition and modal state.
- `src/components/`: cards, lists, forms, modals, and panels.
- `src/composables/`: services, workspaces, Git, theme, logs, usage, and notifications.
- `src/types/`: shared TypeScript contracts.
- `src/style.css`: visual tokens, themes, and global styles.

## Backend

- `commands.rs`: commands exposed to the interface.
- `config.rs`: persistent configuration.
- `process.rs`: process creation and control.
- `runtime.rs`: runtime state and reconciliation.
- `metrics.rs`: CPU, RAM, and processes.
- `health.rs`: HTTP/TCP/process health checks.
- `logs.rs` and `dashboard_log.rs`: service and application logs.
- `git.rs`: Git operations.
- `workspaces.rs`: workspaces and service order.
- `dependencies.rs` and `environment.rs`: environment detection.

## Data

- Configuration: `%APPDATA%\CardumenDeck\config.json`
- Runtime state: `%APPDATA%\CardumenDeck\runtime.json`
- Logs: `%LOCALAPPDATA%\CardumenDeck\logs\`
- Legacy migration: copies `config.json`, `runtime.json`, and the dashboard log from `ServiceDashboard` when `CardumenDeck` does not exist yet.

## Trust boundaries

- The application runs with the Windows user's privileges.
- Service commands run with those same privileges.
- There is no security sandbox.
- The Job Object is used for process management, not isolation.
- Commands, repositories, URLs, and configured endpoints are not trusted by default.
- Configuration and logs are not encrypted.

## Process lifecycle

The application creates local processes, records their PID and state, and reconciles orphaned processes at startup. Services can be adopted if they are still alive.

## Health checks

The monitor runs in the background and updates health state. Checks can be:

- HTTP: response below 500.
- TCP: connection to the port.
- Process: PID alive.

## Network

| Action | Destination | Data sent | When it happens |
|---|---|---|---|
| HTTP health check | Configured URL | `GET` with `User-Agent: CardumenDeck/0.1` | While health checking is active |
| TCP health check | Configured host and port | TCP connection | While health checking is active |
| Git operations | Configured remote | Standard Git data | When the user runs `fetch`, `pull`, or `push` |
| Open URL | Default browser | The requested URL | When the user opens a URL |
| WebView2/Windows | System services | Data according to system settings | Outside the application's control |

The application does not add credentials to HTTP health checks. Git may use Git Credential Manager or another system credential helper; the application does not store those credentials.

## Data at rest

| Data | Location | Encrypted | Notes |
|---|---|---|---|
| Configuration | `%APPDATA%\CardumenDeck\config.json` | No | Includes environment variables |
| Runtime state | `%APPDATA%\CardumenDeck\runtime.json` | No | PIDs, statuses, and paths |
| Logs | `%LOCALAPPDATA%\CardumenDeck\logs\` | No | May contain secrets |
| WebView2 profile | `%LOCALAPPDATA%\CardumenDeck\` | Per system | Theme, view, and preferences |

## Telemetry

Cardumen Deck does not include telemetry, analytics, remote crash reporting, or a backend of its own. Windows, WebView2, Git, and configured endpoints may have their own policies and connections.

See [PRIVACY.en.md](../PRIVACY.en.md) and [SECURITY.en.md](../SECURITY.en.md).

## Security

- The application executes commands configured by the user.
- There is no telemetry or connection to a backend of its own.
- Health checks only contact configured endpoints, and Git contacts the remote when the user runs an operation.
- Releases are signed when SignPath Foundation allows it.
- The window uses a restrictive Content Security Policy (CSP); the theme script lives in `FrontEnd/public/theme-init.js`.
- Secrets and certificates are not committed.
