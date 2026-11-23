# Backlog

## Orb

1. Changes colors based on emotion.
2. Animation should change when speaking to match voice.

## Model Configs

Create unified model config type that is configured via environment variables
for binary containers.

Add configurable chat decoding inputs for temperature, top-p, top-k, and seed,
while keeping greedy decoding as the default.

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

1. Move message content mapping from exec to the state service.
2. Process and parse agent messages the same way as user messages.

## Chat

1. message replies
2. chat threads
3. Replace the static context-window meter with a live context-usage indicator.
4. Add slash commands to the chat composer.
5. Render structured assistant UI blocks, including charts and inputs.

## Frontend

1. Add a Cmd+Shift+P command palette.
2. Replace the dev-tools overlay with draggable, resizable command windows.
3. Add universal search UI across chats, messages, memories, and artifacts.

## Integrations

1. Add MCP plugin support.
2. Add OpenAI-compatible Responses API support.

## Logs/Tracing

Add a new logs view that is a live trace/log stream that can be used to
trace actions to linked stored data.
