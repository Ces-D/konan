# Konan

Konan is a system for receiving messages and media from a client (like a phone or laptop), processing them, and printing them on a Rongta RP326 receipt printer. The goal is to create a seamless interface for physical printing.

## Architecture

### Frontend

**Goal**: A public interface to trigger commands to be printed.

A Svelte web client written in Typescript exposing:

- WYSIWYG editor with a narrow scope of printer friendly extensions
- Buttons that trigger pre-baked printer actions

**Method**
The web client will be hosted as a static site in an S3 bucket.

### Bridge

**Goal**: Exposing the raspberry pi to "outside" actions while maintaining security

Three AWS technologies will power the bridge between a raspberry pi connectd to a private network and the public internet.

1. _API Gateway_ - Provides endpoints for the frontend to `POST` to
2. _Lambda functions_ - Provides handlers for API endpoints. Triggers publication of IoTCore events
3. _IoTCore_ - Is pub/sub for the raspberry pi. Publishing events triggered from the Lambda functions. Subscribing the Pi to take action based on the message.

## Rongta Model RP326

> 📌 A “code page” controls how byte values map to printed characters — especially accented letters and special symbols.
> Different printers may support slightly different sets, but these are the most common.

[ Manual ](https://www.cleancss.com/user-manuals/2AD6G/-RP326-USE)

### Common ESC/POS Code Pages (with examples)

| Codepage                   | Region / Language Focus          | Examples of Characters & Symbols (besides basic A–Z) |
| -------------------------- | -------------------------------- | ---------------------------------------------------- |
| **PC437 (USA)**            | Original IBM PC / U.S. English   | ñ, ç, £, ¥, §, ±, √, ░▒▓                             |
| **Katakana**               | Japanese Katakana symbols        | ｱ ｲ ｳ ｴ ｵ ｶ ｷ ﾀ ﾅ ﾏ (half-width)                     |
| **PC850 (Multilingual)**   | Western European (broad)         | é, è, ä, ö, ü, à, ç, ñ, ß                            |
| **PC860**                  | Portuguese                       | ã, õ, ê, ç, Â, Ô                                     |
| **PC863**                  | Canadian French                  | é, è, ê, à, ç, û                                     |
| **PC865**                  | Nordic / Scandinavian            | Ø, ø, Å, å, Æ, æ                                     |
| **WPC1252 (Windows-1252)** | Western Europe (modern default)  | €, é, è, ä, ö, ü, ñ, œ, Œ                            |
| **PC866**                  | Cyrillic (Russian, etc.)         | д, ж, й, п, ч, ш, Я, Ю                               |
| **PC852**                  | Central/Eastern Europe (Latin-2) | ł, ą, ż, š, č, ř, ť, ě                               |
| **PC858**                  | PC850 + Euro update              | €, é, ç, ñ, ä, ö, ü                                  |

> 👉 **PC437, PC850, and WPC1252** are the most common in POS software.
> 👉 **WPC1252** is often the safest modern choice if you print European accents **and** the Euro sign (€).

---

### What “symbols” really change between code pages

Changing the code page mainly affects:

- **Accented letters**
  (é è ê à ñ ç ä ö ü ø å ł ą …)
- **Currency signs**
  ($, £, ¥, €, sometimes ₫, ₤)
- **Language-specific letters**
  (Å, Ø, Æ in Nordic; ł, ň, ž in Central Europe; Cyrillic in PC866)
- **Legacy box-drawing characters** in PC437
  (useful for receipt borders)

Example:
If your app sends the byte `0x80`…

- in **PC437**, it prints `Ç`
- in **WPC1252**, it prints `€`

Same byte — **different characters** depending on code page.

---

### Quick reference: When to choose which

- ✅ **English only** → PC437 or WPC1252
- ✅ **Spanish, French, German, Italian, etc.** → WPC1252 or PC850
- ✅ **Portuguese (Brazil/Portugal)** → PC860 or WPC1252
- ✅ **Nordic (Norway/Sweden/Denmark)** → PC865
- ✅ **Central/Eastern Europe** → PC852
- ✅ **Russian/Cyrillic** → PC866
- ✅ **Japanese Katakana text labels** → Katakana

---
