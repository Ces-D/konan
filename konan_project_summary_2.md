# Konan Project – No-Backend, Smart Local Printing

## Project Overview

- **Project Name:** Konan
- **Purpose:** Accept user-authored or AI-enhanced messages from iPhone or Mac, print them to a Rongta RP326 receipt printer via static IP over Ethernet, or store locally until the printer is reachable.
- **No backend:** All processing and queuing is handled on-device — no AWS, no SQS, no Lambda.
- **Output formats:** ASCII-only (pure text or ASCII art).
- **Media support:** Text, small photos, video, audio (small assets processed immediately by AI; not stored).
- **Platforms:** macOS (CLI) and iOS (minimal share app).
- **Distribution:** Personal use; sideloaded on iOS or signed via Apple Developer account.

---

## Printer Integration

- **Model:** Rongta RP326 (ESC/POS protocol, USB/Ethernet).
- **Connectivity:** Ethernet with static IP (e.g., `192.168.1.50`), port 9100 (RAW TCP).
- **Printing Protocol:** TCP socket send of ASCII lines + ESC/POS partial cut (`[0x1D, 0x56, 0x41]`).
- **No polling capability:** Printer cannot fetch jobs; devices must push to it.

---

## Smart Network-Based Printing

### Decision flow:

1. Receive print request (via CLI on Mac, or URL scheme on iPhone).
2. Check network:
   - **macOS:** TCP probe to printer IP:port; optional SSID check via `airport -I`.
   - **iOS:** TCP probe + optional SSID allow-list (requires Location permission).
3. If reachable → print immediately.
4. If not reachable → store in **Outbox** for later.
5. Flush Outbox manually or automatically when network is available.

---

## Outbox

- **Role:** Local queue of print jobs that couldn’t be sent immediately.
- **Job fields:** `id`, `created_at`, `mode` (`echo`/`enhance`), `width_cols`, `lines[]`, `status` (`pending`/`printed`/`failed`), `attempts`, `last_error`.
- **Flush behavior:** Retry all pending jobs until printed or marked failed.
- **Storage:**
  - **macOS:** SQLite DB in `~/Library/Application Support/Konan`.
  - **iOS:** Core Data or SQLite in app sandbox.

---

## Mac CLI (Primary Logic)

- **Language:** Rust (`tokio`, `rusqlite`, `clap`).
- **Commands:**
  - `konan print "text..." [--width 42] [--mode echo|enhance]`
  - `konan flush` – Send all pending jobs.
  - `konan list` – Show pending/printed jobs.
  - `konan config set printer.ip 192.168.1.50` (and other settings).
- **Configurable:** printer IP/port, width, SSID allow-list, cut type.
- **Auto-flush:** LaunchAgent runs `konan flush` periodically or on login.

---

## iOS App (Lightweight Router)

- **Purpose:** Accept shared content from any app via Shortcut → URL scheme.
- **URL Scheme:** `konan://print?text=...&mode=echo|enhance&width=42`, `konan://flush`.
- **Actions:**
  - Probe + print immediately if reachable.
  - Enqueue in local Outbox if not.
- **UI:** Minimal — printer settings + Outbox list.
- **Automation:** Shortcuts for “Send to Konan” and “Flush Outbox” (triggered on Wi-Fi join).

---

## Message Modes & Handling

- **Modes:**
  - `echo`: Print exactly as provided.
  - `enhance`: Send to AI for ASCII-friendly formatting (32/42-col wrap, ≤1–2 KB).
- **ASCII rendering:** Strip/replace wide glyphs, wrap to set width, prepend optional header.

---

## Automation

- **macOS:** LaunchAgent for periodic flush; optional Shortcut to trigger `konan://flush`.
- **iOS:** Shortcuts to print or flush from Share Sheet or automation triggers.

---

## Development Plan & Timeline (Single Developer, ~5 Days)

| Platform  | Task                                                         | Effort |
| --------- | ------------------------------------------------------------ | ------ |
| Shared    | Define URL scheme (`print`/`flush`), job schema, ASCII rules | 0.25d  |
| macOS CLI | Scaffold Rust CLI, config mgmt, logging                      | 0.5d   |
|           | ASCII wrap/sanitizer                                         | 0.5d   |
|           | TCP probe + print                                            | 0.5d   |
|           | SQLite Outbox (enqueue/list/flush)                           | 0.75d  |
|           | SSID check (`airport -I`)                                    | 0.25d  |
|           | LaunchAgent config/docs                                      | 0.25d  |
| iOS App   | Register URL scheme, handle `print`/`flush`                  | 0.25d  |
|           | Probe + print or enqueue (Core Data)                         | 0.5d   |
|           | Minimal UI: settings + Outbox list                           | 0.5d   |
|           | Shortcuts integration                                        | 0.25d  |
| QA        | Test offline/online, wrong SSID, wrap rules, cut command     | 0.5d   |

---

## Key Advantages of This Design

- **No cloud dependencies** – zero AWS cost, full control.
- **Resilient** – jobs queue locally if offline; flush later.
- **Unified UX** – same logic across Mac and iPhone.
- **Low maintenance** – static binaries, minimal iOS app footprint.
- **Privacy** – no data leaves the devices except to the printer or AI API (if used).
