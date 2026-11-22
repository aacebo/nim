# Backlog

## Orb

1. Changes colors based on emotion.
2. Animation should change when speaking to match voice.

## Model Configs

Create unified model config type that is configured via environment variables
for binary containers.

## Errors

Error nesting/rollup.

## Worker Healthchecks

Add http endpoints for metrics/health to workers for observability.

## Inference

Add inference loop to `nim-exec` that uses qwen starting weights then performs live
weight updates.

## Platt Scaling

Add platt scaling weights to models in database and read/write to them.

## Debt

1. Message::parent_id -> Message::reply_to_id
2. move message content mapping from exec -> state service
3. agent messages should be processed and parsed the same way we do user messages
