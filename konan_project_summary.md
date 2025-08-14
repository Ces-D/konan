# Konan Project - Chat Summary & Categorization

## Project Overview

- **Project Name:** Konan
- **Purpose:** Accept requests from phone/laptop in a chat-like interface, process them (optionally via AI enhancement), and print via receipt printer (Rongta RP326, ESC/POS).
- **Output formats:** Pure text and ASCII art only (no ESC/POS graphics in MVP).
- **Media support:** Text, photos, video, audio (small assets passed directly to AI, not stored).
- **Architecture preference:** Local agent + cloud backend (Lambda + S3 + SQS FIFO).

## Printer & Agent

- **Printer Model:** Rongta RP326 (ESC/POS protocol, USB/Ethernet).
- **Connectivity:** Supports Ethernet with static IP; raw TCP printing on port 9100 works, but printer **cannot poll** for jobs itself—requires a LAN device to push data.
- **Agent Responsibilities:** Poll SQS FIFO for print jobs, fetch or receive ASCII payloads, send to printer via TCP, cut paper.
- **Agent Implementation:** Node.js/TypeScript primary; optional Python fallback.
- **Printing Libraries:** `node-thermal-printer`, `escpos`, ASCII rendering tools (`image-to-ascii`, `sharp` for preprocessing, `qrcode` for QR if needed later).
- **Rendering:** Text directly; media converted to ASCII art or ASCII summaries (no QR/bitmap in MVP).

## Backend Architecture (Lambda-based)

- **Serverless Components:** API Gateway (HTTP API), AWS Lambda, S3 (optional for uploads), SQS FIFO, DynamoDB (message metadata).
- **Job Flow:** Client sends text or media → Lambda processes (echo or enhance) → renders ASCII output → enqueues job to SQS FIFO → Agent prints.
- **Upload Flow:** For larger/stored files, presigned URL to S3 → finalize → Lambda enqueues job (not used for MVP small assets).
- **Queue Delivery:** SQS FIFO long-polling by Agent (20s wait), single `MessageGroupId` for strict print order.
- **Authentication:** API key now; potential Cognito or IAM roles later.
- **Cost Efficiency:** Pay-per-use Lambda, SQS, S3; minimal constant cost.

## Message Types & Handling

- **Unified API:** Single `POST /messages` endpoint for all message types (`text`, `photo`, `video`, `audio`).
- **Client-specified mode:** `echo` (print as-is), `enhance` (AI-enhanced output), or `auto` (Lambda decides).
- **Enhancement:** When enabled, Lambda calls an AI model to format/condense content to ASCII within 1–2 KB, avoiding wide glyphs/emojis.
- **Media Handling:** Small assets (≤5 MB) sent inline to Lambda → passed to AI → summarized/described in ASCII. Assets not stored in MVP.
- **Print Payload:** Always ASCII text, wrapped to 32 or 42 columns with header/footer. Optional cut command at end.
- **No idempotency** for MVP; at-least-once delivery acceptable.

### Example Flow

1. Client sends `/messages` with `mode=enhance` and text or inline media.
2. Lambda validates, processes, formats to ASCII.
3. Lambda enqueues print job JSON to SQS FIFO.
4. Agent polls queue, sends payload to `tcp://<printer-ip>:9100`, prints.

## Decisions Made in This Conversation

1. **Client-controlled enhancement mode**
   - **Reason:** Allows user to choose when to use AI (cost control, flexibility).
2. **Small assets passed directly to AI, not stored**
   - **Reason:** Avoids S3 complexity in MVP; suits quick-turnaround printing.
3. **ASCII-only printing**
   - **Reason:** Simpler rendering, avoids ESC/POS bitmap complexity, maximizes compatibility and readability.
4. **Single FIFO queue for all jobs**
   - **Reason:** One printer in MVP; FIFO ensures strict ordering.
5. **No idempotency for MVP**
   - **Reason:** Lower complexity; reprints on retries acceptable in early stage.
6. **Lambda cannot print directly to printer**
   - **Reason:** RP326 is on private LAN; Lambda has no network route without complex VPN/tunnel setup.
   - **Resolution:** Keep a minimal LAN agent to receive jobs from SQS and push via TCP 9100 to static printer IP.

## Development Plan

- Use AWS CDK/SST for infrastructure deployment.
- Implement minimal API endpoints: `/messages`, `/upload-request`, `/finalize`, `/acks` (optional).
- Implement SQS FIFO job schema for ASCII-only payloads.
- Parallel development: Infrastructure + Agent printing pipeline.
- Testing: End-to-end with text and photo-to-ASCII.
- Future Enhancements: Multi-user auth, richer media processing, QR/bitmap support, potential cloud-print capable hardware.

## Trade-offs & Design Decisions

- **Node.js/TypeScript** chosen for both Lambda and Agent for uniform codebase.
- **Local rendering & printing** for MVP (flexibility, cost efficiency).
- **SQS FIFO** for ordering, low cost, and decoupling cloud logic from LAN printing.
- **ASCII rendering** for printer-friendly, simple output.
- **LAN agent** is required; direct Lambda-to-printer path is not feasible without complex networking.
