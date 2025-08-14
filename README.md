# Konan

Konan is a system for receiving messages and media from a client (like a phone or laptop), processing them, and printing them on a Rongta RP326 receipt printer. The goal is to create a seamless chat-like interface for physical printing.

## Overview

The project is designed with a hybrid architecture: a cloud-based backend for processing and queuing, and a local agent for printer communication. This decouples the core logic from the local network and allows for a scalable, event-driven system.

The primary output format is ASCII text, ensuring compatibility and simplicity. Media files (photos, video, audio) are not printed directly but are converted into ASCII art or text summaries by an AI service.

## Architecture

The system is composed of two main parts:

1.  **Cloud Backend (AWS Serverless)**:
    *   **API Gateway**: Provides an HTTP endpoint (`/messages`) to receive requests.
    *   **AWS Lambda**: The core processing engine. It handles incoming requests, interacts with AI services for content enhancement, and enqueues print jobs.
    *   **Amazon SQS (FIFO)**: A First-In, First-Out queue to manage print jobs, ensuring they are processed in the correct order.
    *   **Amazon S3**: Used for handling file uploads via presigned URLs (for larger assets, not used in the MVP for small, inline media).

2.  **Local Agent**:
    *   A lightweight application running on the same local network as the printer.
    *   **Responsibilities**:
        *   Polls the SQS queue for new print jobs.
        *   Sends the job payload (ASCII text) directly to the Rongta printer via a raw TCP connection (`<printer-ip>:9100`).
        *   Handles printer-specific commands like cutting the paper.
    *   **Technology**: Intended to be implemented in Node.js/TypeScript.

## Features

- **Multiple Input Modes**: Clients can specify how a message is handled:
    - `echo`: Print the text as-is.
    - `enhance`: Use an AI service to summarize or reformat the content.
    - `auto`: Let the backend decide the best mode.
- **Media-to-ASCII**: Photos, videos, and audio are converted into text-based representations for printing.
- **Strict Print Ordering**: A single SQS FIFO queue ensures that jobs are printed in the order they are received.
- **Local Printing**: The local agent bridges the gap between the cloud and the LAN-connected printer.

## Development Status

The project is currently in the initial setup phase. The architecture has been defined, and the next step is to implement the cloud backend and the local print agent.
