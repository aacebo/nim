# Neural Intelligence Mesh

A continual-learning multi-modal language model.

This projects goal is to model how the human brain functions as closely as possible
using small models, large models, and equivalent transports.

## What Does This Mean?

1. Small/large models are used to mimick how different regions of the brain work.
2. Those different regions communicate with one another as a DAG in the same way our brains do.
3. Data is modelled/stored as similarly to how the human brain stores it as possible.

## Why?

I've long since been curious if by modeling a system like this after the human brain we could create
a more intelligent, self sufficient, and cost effective/performant intelligence.

Rephrased, can you make/train a general purpose intelligence model built on a network of 
smaller models more speicific to one given task?

```mermaid
flowchart TD
    A["Input tokens<br/>Token embeddings"]
    B["NIM neural network<br/>Learned layers + parameters"]
    C["Memory integration<br/>Retrieve and integrate relevant memories"]
    D["Prediction head<br/>Next-token probability distribution"]
    E["Token sampling<br/>Select and append next token"]

    A --> B
    B --> C
    C --> D
    D --> E
    E -.->|Autoregressive feedback| A

    classDef default fill:#17191c,stroke:#657380,color:#e5e7eb,stroke-width:1.5px
    classDef memory fill:#dbe8f6,stroke:#71a8db,color:#214e74,stroke-width:2px

    class C memory
```

```mermaid
flowchart TD
    Q["Qwen pretrained weights"]
    LM["NIM language model<br/>Inference + active adapters"]

    Q --> LM

    LM --> A["Awake cycle<br/>Observe + remember<br/>Optional rapid adaptation"]
    LM --> S["Sleep cycle<br/>Replay + train<br/>Evaluate + consolidate"]

    A --> M["Persistent memory + replay buffer<br/>Experiences, feedback, provenance"]
    S --> M

    M -.->|Feedback loop| LM
    M --> P["Validated adapter promotion / rollback"]

    classDef model fill:#4984bb22,stroke:#4984bb,color:#4984bb
    classDef awake fill:#3b9e8a22,stroke:#3b9e8a,color:#3b9e8a
    classDef sleep fill:#aa834822,stroke:#aa8348,color:#aa8348
    classDef neutral fill:#8395aa22,stroke:#8395aa,color:#8395aa

    class LM model
    class A awake
    class S sleep
    class Q,M neutral
    class P awake
```
