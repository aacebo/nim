# Gateway

The ingress service. It exposes the HTTP and web UI surface, persists chats,
messages, and artifacts, publishes workflow signals, and forwards chat, message,
and memory updates to connected sockets.

## Service Mesh

Arrows show logical signal flow. This service is highlighted.

```mermaid
flowchart TD
    GW["nim-gateway"]
    TXT["nim-text"]
    AUD["nim-audio"]
    VIS["nim-vision"]
    ENC["nim-encode"]
    EXE["nim-exec"]
    SLP["nim-sleep"]

    GW -->|artifact.create| TXT
    GW -->|artifact.create| AUD
    GW -->|artifact.create| VIS
    GW -->|message.create / memory.recall| ENC
    TXT -->|artifact.analyze| ENC
    AUD -->|artifact.analyze| ENC
    VIS -->|artifact.analyze| ENC
    ENC -->|message.analyze| EXE

    classDef gateway fill:#172554,stroke:#38bdf8,color:#e0f2fe,stroke-width:2px
    classDef service fill:#172033,stroke:#64748b,color:#e2e8f0
    classDef planned fill:#1f2937,stroke:#475569,color:#94a3b8,stroke-dasharray: 4 4
    classDef focus fill:#164e63,stroke:#22d3ee,color:#ecfeff,stroke-width:3px

    class TXT,AUD,VIS,ENC,EXE service
    class SLP planned
    class GW focus
```

## Responsibilities

- Serves `/health`, search, artifact, memory, chat, and web-view routes.
- Routes text, audio, and visual artifacts to their modality worker.
- Publishes `nim.chat.*`, `nim.message.*`, and `nim.memory.recall` signals.

## Current limits

- Unknown artifact types are published but have no worker.
- The gateway does not execute inference.
