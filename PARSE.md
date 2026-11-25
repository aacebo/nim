| Project | Category | Location | What it parses |
|---|---|---|---|
| nim-gateway | User message content | [Chat composer](bins/nim-gateway/templates/chat/chat.js#L434) | Scans typed text for HTTP URLs and JSON objects or arrays, producing structured content parts. Skips fenced and inline code. |
| nim-gateway | User message submission | [Chat submission](bins/nim-gateway/templates/chat/chat.js#L583) | Adds artifact parts to the parsed composer content before sending it. |
| nim-ai | Model output | [Qwen2 template](crates/nim-ai/src/models/qwen2/template.rs#L387) | Extracts reasoning and tool-call blocks from generated text; parses tool-call JSON and arguments. |
| nim-ai | Model output | [Qwen3 template](crates/nim-ai/src/models/qwen3/template.rs#L388) | Performs the equivalent Qwen3 output parsing. |
| nim-chat | Parsing utility | [nim-chat parser](crates/nim-chat/src/parser.rs#L1) | Supplies the string-scanning operations used by both Qwen templates. |
| nim-ai | Model output | [OpenAI client](crates/nim-ai/src/clients/openai/client.rs#L108) | Extracts content and reasoning from provider JSON and parses tool-call argument strings as JSON. Plain content is trimmed and kept as a string. |
| nim-ai | Model task output | [OpenAI JSON path](crates/nim-ai/src/clients/openai/client.rs#L33) | Parses schema-constrained response text as JSON for remote classification and entity extraction. |
| nim-gateway | Display | [Chat transcript](bins/nim-gateway/templates/chat/transcript/transcript.js#L60) | Parses message text as Markdown for display. |
| nim-gateway | Display | [Message detail view](bins/nim-gateway/templates/message/message.js#L42) | Separately parses message text as Markdown for display. |
| nim-gateway | API boundary | [New-chat endpoint](bins/nim-gateway/src/routes/chats/create.rs#L5) and [existing-chat endpoint](bins/nim-gateway/src/routes/chats/messages/create.rs#L11) | Deserialize request JSON into `ChatMessageContent`; they do not scan submitted strings for URLs or embedded JSON. |
| nim-state | Persistence boundary | [State message loading](crates/nim-state/src/types/message.rs#L109) | Deserializes stored JSON into `ChatMessageContent`; it does not parse free-form text. |
| nim-runtime | Runtime resolution | [Runtime rendering](crates/nim-runtime/src/render.rs#L97) | Parses artifact IDs and URLs while resolving structured parts; it does not parse free-form text. |
