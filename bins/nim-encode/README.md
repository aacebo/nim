# Encode

Enriches artifacts, memories, messages, and chats after modality processing.

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

    class GW gateway
    class TXT,AUD,VIS,EXE service
    class SLP planned
    class ENC focus
```

## Responsibilities

- Consumes `nim.artifact.analyze`, `nim.memory.analyze`, `nim.memory.recall`, and `nim.message.{create|analyze}`.
- Computes artifact and memory confidence, salience, and strength.
- Summarizes and embeds memories and chats; enriches messages with summaries and annotations.
- Emits `nim.memory.encode`, `nim.memory.update`, `nim.message.update`, `nim.message.analyze`, and `nim.chat.update`.

## Current limits

`nim.memory.encode` has no consumer yet.
