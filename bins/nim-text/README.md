# Text

Processes text artifacts into summaries, embeddings, annotations, and linked
working memories.

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
    class AUD,VIS,ENC,EXE service
    class SLP planned
    class TXT focus
```

## Responsibilities

- Consumes `nim.text.artifact.create`.
- Summarizes and embeds text; extracts entities, keywords, and label annotations.
- Creates or updates a linked working memory.
- Emits `nim.artifact.create`, `nim.artifact.analyze`, and `nim.memory.{create|update}`.
