customElements.define("nim-chat", class extends HTMLElement {
    createView(host, signal) {
        const orbScene = document.querySelector(".orb-scene");
        const actionPanel = host.querySelector("#action-panel");
        const actionView = actionPanel.closest(".action-view");
        const chatForm = host.querySelector("#chat-form");
        const chatInput = host.querySelector("#chat-input");
        const chatSend = host.querySelector("#chat-send");
        const chatAttach = host.querySelector("#chat-attach");
        const chatFiles = host.querySelector("#chat-files");
        const chatAttachments = host.querySelector("#chat-attachments");
        const chatAttachmentDrawer = host.querySelector("#chat-attachment-drawer");
        const chatDrawer = host.querySelector("#chat-drawer");
        const chatDrawerToggle = host.querySelector("#chat-drawer-toggle");
        const chatDrawerScrim = host.querySelector("#chat-drawer-scrim");
        const chatHistory = host.querySelector("#chat-history");
        const chatNew = host.querySelector("#chat-new");
        const pendingAttachments = [];
        // Leave room for multipart headers within the gateway's 10 MiB request limit.
        const maxAttachmentSize = 10 * 1024 * 1024 - 64 * 1024;
        const textExtensions = /\.(txt|md|csv|json|log|ya?ml)$/i;
        const attachmentPaths = {
            image: "M3 3h18v18H3z M3 17l6-6 5 5 3-3 4 4 M16 7h.01",
            audio: "M9 18V5l11-2v13 M9 8l11-2 M9 18c0 2-6 3-6 0s6-2 6 0 M20 16c0 2-6 3-6 0s6-2 6 0",
            text: "M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8 M8 16h8",
        };
        const chatError = host.querySelector("#chat-error");
        const chatHistoryStatus = host.querySelector("#chat-history-status");
        const initialState = host.querySelector(".action-view").dataset;
        const initialChatView = JSON.parse(initialState.chat);
        const initialChat = initialChatView?.chat ?? null;
        const initialMessages = initialChatView?.messages ?? [];
        const chatMessages = new Map(initialMessages.map((message) => [message.id, message]));
        const signalDecoder = new TextDecoder();
        const historyLimit = 100;
        const reasoningBackground = "radial-gradient(ellipse var(--orb-hue-inner-x, 50vmax) var(--orb-hue-inner-y, 50vmax) at var(--orb-center-x, 50%) var(--orb-center-y, 50%), rgb(15 58 126 / 42%), transparent), #02070f";
        let reasoningOverlay = orbScene.querySelector("#reasoning-background");

        if (!reasoningOverlay) {
            reasoningOverlay = document.createElement("div");
            reasoningOverlay.id = "reasoning-background";
            reasoningOverlay.style.cssText = `position:absolute;z-index:0;inset:0;pointer-events:none;background:${reasoningBackground};opacity:0;transition:opacity 900ms ease`;
            orbScene.prepend(reasoningOverlay);
        }
        for (const child of orbScene.children) {
            if (child !== reasoningOverlay) {
                child.style.position = "relative";
                child.style.zIndex = "1";
            }
        }

        let chatId = initialChat?.id;
        let historyCursor;
        let historyLoaded = false;
        let historyLoading = false;
        let historyError = false;
        let chatErrorTimer;
        let awaitingResponse = false;
        let activity;
        let activityTimer;
        let orbCenterFrame;
        const drawerPreferenceKey = "nim.chat.drawer-open";
        const pushDrawer = matchMedia("(min-width: 961px)");
        const orbTrigger = orbScene.querySelector("#orb-trigger");
        const orbResizeObserver = new ResizeObserver(syncOrbCenter);
        orbResizeObserver.observe(orbTrigger);

        function syncOrbCenter() {
            const sceneBounds = orbScene.getBoundingClientRect();
            const orbBounds = orbTrigger.getBoundingClientRect();
            orbScene.style.setProperty("--orb-center-x", `${orbBounds.left - sceneBounds.left + orbBounds.width / 2}px`);
            orbScene.style.setProperty("--orb-center-y", `${orbBounds.top - sceneBounds.top + orbBounds.height / 2}px`);
            orbScene.style.setProperty("--orb-hue-inner-x", `${orbBounds.width * 0.75}px`);
            orbScene.style.setProperty("--orb-hue-inner-y", `${orbBounds.height * 0.75}px`);
            orbScene.style.setProperty("--orb-hue-outer-x", `${orbBounds.width * 1.1}px`);
            orbScene.style.setProperty("--orb-hue-outer-y", `${orbBounds.height * 1.1}px`);
        }

        function followOrbCenter() {
            cancelAnimationFrame(orbCenterFrame);
            const end = performance.now() + 400;

            function frame(now) {
                syncOrbCenter();
                if (now < end) orbCenterFrame = requestAnimationFrame(frame);
            }

            orbCenterFrame = requestAnimationFrame(frame);
        }

        function syncOrbScale() {
            const viewportWidth = window.innerWidth;
            const drawerWidth = actionView.classList.contains("drawer-open") && pushDrawer.matches
                ? chatDrawer.getBoundingClientRect().width
                : 0;
            orbScene.style.setProperty("--chat-orb-scale", String(0.66125 * (viewportWidth - drawerWidth) / viewportWidth));
            syncOrbCenter();
        }

        function saveDrawerPreference(open) {
            try { localStorage.setItem(drawerPreferenceKey, String(open)); } catch {}
        }

        function setDrawerOpen(open, save = true) {
            actionView.classList.toggle("drawer-open", open);
            orbScene.classList.toggle("chat-drawer-open", open);
            orbScene.classList.toggle("chat-drawer-push", open && pushDrawer.matches);
            syncOrbScale();
            followOrbCenter();
            chatDrawer.inert = !open;
            chatDrawerToggle.setAttribute("aria-expanded", String(open));
            chatDrawerToggle.setAttribute("aria-label", open ? "Close chat history" : "Open chat history");
            if (save) saveDrawerPreference(open);
        }

        function closeDrawer() {
            saveDrawerPreference(actionView.classList.contains("drawer-open"));
            setDrawerOpen(false, false);
        }

        function dismissOverlayDrawer() {
            if (pushDrawer.matches || !actionView.classList.contains("drawer-open")) return false;
            setDrawerOpen(false);
            return true;
        }

        function restoreDrawer() {
            let drawerOpen;
            try { drawerOpen = localStorage.getItem(drawerPreferenceKey); } catch {}
            setDrawerOpen(drawerOpen === null || drawerOpen === undefined ? pushDrawer.matches : drawerOpen === "true", false);
        }

        function chatLabel(chat) {
            const content = chat.last_message?.content;
            return messageLabel(content, chat.summary);
        }

        function messageLabel(content, fallback = "") {
            const text = typeof content === "string" ? content : Array.isArray(content)
                ? content.filter((part) => part?.type === "text").map((part) => part.text).join(" ") : "";
            return (text || fallback || "New chat").replace(/\s+/g, " ").trim();
        }

        function updateActiveChatPreview(message) {
            const link = chatHistory.querySelector('a[aria-current="page"]');
            if (link) link.textContent = messageLabel(message.content);
        }

        function renderChatList(chats, append = false) {
            const rows = chats.map((chat) => {
                const item = document.createElement("li");
                const link = document.createElement("a");
                link.href = `/web/chat/${encodeURIComponent(chat.id)}`;
                link.textContent = chatLabel(chat);
                if (chat.id === chatId) link.setAttribute("aria-current", "page");
                item.append(link);
                return item;
            });
            if (append) chatHistory.append(...rows);
            else chatHistory.replaceChildren(...rows);
        }

        async function loadChatList() {
            const query = new URLSearchParams({ limit: "50" });
            const response = await fetch(`/chats?${query}`, { cache: "no-store" });
            if (!response.ok) return;
            const page = await response.json();
            renderChatList(page.data);
        }

        chatDrawerToggle.addEventListener("click", () => setDrawerOpen(!actionView.classList.contains("drawer-open")), { signal });
        chatDrawerScrim.addEventListener("click", () => setDrawerOpen(false), { signal });
        chatNew.addEventListener("click", () => { window.location.assign("/web/chat"); }, { signal });
        pushDrawer.addEventListener("change", () => {
            orbScene.classList.toggle("chat-drawer-push", actionView.classList.contains("drawer-open") && pushDrawer.matches);
            syncOrbScale();
        }, { signal });
        window.addEventListener("resize", syncOrbScale, { signal });
        signal.addEventListener("abort", () => {
            cancelAnimationFrame(orbCenterFrame);
            orbResizeObserver.disconnect();
            orbScene.classList.remove("chat-drawer-open", "chat-drawer-push");
        }, { once: true });
        restoreDrawer();
        void loadChatList();

        function updateTranscriptInset() {
            actionView?.style.setProperty("--chat-composer-height", `${Math.ceil(actionPanel.getBoundingClientRect().height)}px`);
        }

        const composerObserver = new ResizeObserver(updateTranscriptInset);
        composerObserver.observe(actionPanel);
        signal.addEventListener("abort", () => composerObserver.disconnect(), { once: true });
        requestAnimationFrame(updateTranscriptInset);

        function resizeChatInput() {
            chatInput.style.height = "auto";
            chatInput.style.height = `${Math.min(chatInput.scrollHeight, 144)}px`;
            chatInput.style.overflowY = chatInput.scrollHeight > 144 ? "auto" : "hidden";
        }

        chatInput.addEventListener("input", resizeChatInput, { signal });
        chatInput.addEventListener("keydown", (event) => {
            if (event.isComposing || event.key !== "Enter" || event.shiftKey) return;
            event.preventDefault();
            void submitChatMessage();
        }, { signal });

        async function openMessageView(id, trigger) {
            const response = await fetch(`/web/views/messages/${encodeURIComponent(id)}`);
            if (!response.ok) throw new Error("Unable to load message details");
            const documentView = new DOMParser().parseFromString(await response.text(), "text/html");
            const template = documentView.getElementById("message-template");
            if (!template) throw new Error("Message details are unavailable");
            document.getElementById("message-template")?.remove();
            document.querySelector("nim-message")?.remove();
            document.body.append(template);
            await Promise.all([loadAsset("message/message.css?v=32", "css"), loadAsset("message/message.js?v=26", "js")]);
            const view = document.createElement("nim-message");
            document.body.append(view);
            view.open(trigger);
        }

        host.addEventListener("nim-message-inspect", (event) => {
            void openMessageView(event.detail.id, event.detail.trigger).catch(() => setChatError("Unable to load message details."));
        }, { signal });
        host.addEventListener("nim-chat-history", () => { void loadChatHistory(); }, { signal });
        host.addEventListener("nim-chat-activity-toggle", (event) => {
            if (activity) activity.expanded = event.detail.expanded;
        }, { signal });

        function setChatError(text) {
            clearTimeout(chatErrorTimer);
            chatError.classList.remove("is-fading");
            chatError.textContent = text;
            if (!text) return;
            chatErrorTimer = setTimeout(() => {
                chatError.classList.add("is-fading");
                chatErrorTimer = setTimeout(() => {
                    chatError.textContent = "";
                }, 300);
            }, 2200);
        }

        function formatElapsed(startedAt) {
            const seconds = Math.max(1, Math.floor((Date.now() - startedAt) / 1000));
            return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
        }

        function setAwaitingResponse(value, discard = false) {
            if (awaitingResponse === value) return;
            awaitingResponse = value;
            orbScene.dataset.reasoning = String(value);
            reasoningOverlay.style.opacity = value ? "1" : "0";
            if (value) {
                activity = { active: true, elapsed: "", expanded: false, startedAt: Date.now(), updates: [] };
                activityTimer = setInterval(() => {
                    activity.elapsed = formatElapsed(activity.startedAt);
                    renderChatTranscript();
                }, 1000);
            } else {
                clearInterval(activityTimer);
                activityTimer = undefined;
                if (discard) activity = undefined;
                else if (activity) {
                    activity.active = false;
                    activity.elapsed = formatElapsed(activity.startedAt);
                    activity.expanded = false;
                }
            }
            renderChatTranscript();
        }

        function updateActivity(update) {
            if (!awaitingResponse || !activity || typeof update?.text !== "string" || !update.text.trim()) return;
            if (Number.isInteger(update.sequence) && activity.updates.at(-1)?.sequence >= update.sequence) return;
            if (activity.updates.at(-1)?.text === update.text) return;
            activity.updates.push({ text: update.text, sequence: update.sequence });
            renderChatTranscript();
        }

        function isChatOpen() {
            const state = Alpine.$data(orbScene);
            return state.hud === "panel" && state.showChatForm;
        }

        function openAction(isChat, title) {
            actionPanel.classList.toggle("chat-panel", isChat);
            renderChatTranscript();
            if (isChat) {
                restoreDrawer();
                Alpine.nextTick(() => {
                    let attempts = 0;
                    const focus = () => {
                        if (!isChatOpen() || attempts++ >= 60) return;
                        if (chatForm.hidden || actionPanel.inert || getComputedStyle(chatInput).visibility !== "visible") {
                            requestAnimationFrame(focus);
                        } else {
                            chatInput.focus({ preventScroll: true });
                        }
                    };
                    requestAnimationFrame(focus);
                });
                void loadChatHistory();
            }
        }

        function mergeChatMessage(message) {
            if (!chatId || message?.chat_id !== chatId || typeof message.id !== "string"
                || typeof message.created_at !== "string" || !Number.isInteger(message.version)) {
                return false;
            }

            const previous = chatMessages.get(message.id);

            if (previous && previous.version >= message.version) {
                return false;
            }

            chatMessages.set(message.id, message);
            return true;
        }

        function renderChatTranscript() {
            host.querySelector("nim-chat-transcript").render({
                open: isChatOpen(), chatMessages, historyLoading, historyError, activity,
            });
        }

        async function loadChatHistory() {
            if (!chatId || historyLoading || historyLoaded && !historyCursor) {
                return;
            }

            historyLoading = true;
            historyError = false;
            chatHistoryStatus.textContent = "Loading history…";

            try {
                const query = new URLSearchParams({ limit: historyLimit });
                if (historyCursor) query.set("cursor", historyCursor);
                const response = await fetch(`/chats/${encodeURIComponent(chatId)}/messages?${query}`);
                if (!response.ok) throw new Error("Chat history request failed");
                const page = await response.json();
                for (const message of page.data) mergeChatMessage(message);
                historyCursor = page.next;
                historyLoaded = true;
            } catch {
                historyError = true;
            } finally {
                historyLoading = false;
                renderChatTranscript();
            }
        }

        socket.binaryType = "arraybuffer";
        socket.addEventListener("open", () => { void loadChatHistory(); }, { signal });
        socket.addEventListener("message", async (event) => {
            try {
                const payload = typeof event.data === "string" ? event.data : signalDecoder.decode(event.data);
                const signal = JSON.parse(payload);
                const response = signal?.subject === "nim.message.response" && signal.content?.chat_id === chatId;
                if (signal?.subject === "nim.message.progress" && signal.content?.chat_id === chatId) {
                    updateActivity(signal.content);
                }
                if (mergeChatMessage(signal?.content)) {
                    if (host.querySelector("nim-chat-transcript").messageArtifacts(signal.content.content).length) {
                        await loadArtifactView();
                        void loadChatArtifacts(chatId).then(renderChatTranscript).catch(() => {});
                    }
                    renderChatTranscript();
                }
                if (response) {
                    setAwaitingResponse(false);
                    updateActiveChatPreview(signal.content);
                    void loadChatList();
                }
            } catch {
                return;
            }
        }, { signal });

        function renderAttachments() {
            chatAttachments.hidden = pendingAttachments.length === 0;
            chatAttachmentDrawer.hidden = pendingAttachments.length === 0;
            chatAttachments.replaceChildren(...pendingAttachments.map((attachment) => {
                const item = document.createElement("li");
                const icon = document.createElementNS("http://www.w3.org/2000/svg", "svg");
                const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
                icon.setAttribute("viewBox", "0 0 24 24");
                icon.setAttribute("aria-hidden", "true");
                path.setAttribute("d", attachmentPaths[attachment.kind]);
                icon.append(path);
                const name = document.createElement("span");
                name.className = "attachment-name";
                name.textContent = attachment.file.name;
                const size = document.createElement("span");
                size.className = "attachment-size";
                size.textContent = attachment.file.size < 1024 * 1024
                    ? `${Math.ceil(attachment.file.size / 1024)} KB`
                    : `${(attachment.file.size / (1024 * 1024)).toFixed(1)} MB`;
                const remove = document.createElement("button");
                remove.type = "button";
                remove.textContent = "×";
                remove.setAttribute("aria-label", `Remove ${attachment.file.name}`);
                remove.disabled = chatSend.disabled;
                remove.addEventListener("click", () => {
                    if (chatSend.disabled) return;
                    pendingAttachments.splice(pendingAttachments.indexOf(attachment), 1);
                    renderAttachments();
                    chatAttach.focus();
                });
                item.title = attachment.file.name;
                item.append(icon, name, size, remove);
                return item;
            }));
        }

        function handleChatFiles(files) {
            if (chatSend.disabled) return;
            const errors = [];
            for (const file of files) {
                const kind = file.type.startsWith("image/") ? "image"
                    : file.type.startsWith("audio/") ? "audio"
                    : file.type.startsWith("text/") || textExtensions.test(file.name) ? "text" : null;
                if (!kind) {
                    errors.push(`${file.name}: choose an audio, image, or text file.`);
                } else if (file.size > maxAttachmentSize) {
                    errors.push(`${file.name}: must be smaller than 10 MiB.`);
                } else {
                    pendingAttachments.push({ file, kind, artifactId: null });
                }
            }
            chatFiles.value = "";
            setChatError(errors.join(" "));
            renderAttachments();
        }

        function parseContent(text) {
            const content = [];
            let start = 0;
            let index = 0;

            const pushText = (value) => {
                if (!value) return;
                const previous = content.at(-1);
                if (previous?.type === "text") previous.text += value;
                else content.push({ type: "text", text: value });
            };
            const fencedEnd = (offset) => {
                const lineStart = text.lastIndexOf("\n", offset - 1) + 1;
                if (text.slice(lineStart, offset).trim()) return null;
                const marker = text.slice(offset).match(/^(`{3,}|~{3,})/)?.[1];
                if (!marker) return null;
                const lineEnd = text.indexOf("\n", offset);
                let cursor = lineEnd < 0 ? text.length : lineEnd + 1;
                while (cursor < text.length) {
                    const end = text.indexOf("\n", cursor);
                    const line = text.slice(cursor, end < 0 ? text.length : end).replace(/\r$/, "");
                    if (line.match(new RegExp(`^[ \\t]{0,3}${marker[0]}{${marker.length},}[ \\t]*$`))) {
                        return end < 0 ? text.length : end + 1;
                    }
                    cursor = end < 0 ? text.length : end + 1;
                }
                return text.length;
            };
            const inlineCodeEnd = (offset) => {
                const marker = text.slice(offset).match(/^`+/)?.[0];
                if (!marker) return null;
                const end = text.indexOf(marker, offset + marker.length);
                return end < 0 ? text.length : end + marker.length;
            };
            const jsonEnd = (offset) => {
                const stack = [];
                let quoted = false;
                let escaped = false;
                for (let cursor = offset; cursor < text.length; cursor++) {
                    const character = text[cursor];
                    if (quoted) {
                        if (escaped) escaped = false;
                        else if (character === "\\") escaped = true;
                        else if (character === '"') quoted = false;
                    } else if (character === '"') quoted = true;
                    else if (character === "{" || character === "[") stack.push(character);
                    else if (character === "}" || character === "]") {
                        if (!stack.length) return null;
                        stack.pop();
                        if (!stack.length) return cursor + 1;
                    }
                }
                return null;
            };

            while (index < text.length) {
                const fence = fencedEnd(index);
                if (fence) {
                    index = fence;
                    continue;
                }
                const inlineCode = inlineCodeEnd(index);
                if (inlineCode) {
                    index = inlineCode;
                    continue;
                }
                const boundary = index === 0 || !/[A-Za-z0-9_]/.test(text[index - 1]);
                if (boundary && (text.startsWith("http://", index) || text.startsWith("https://", index))) {
                    const match = text.slice(index).match(/^https?:\/\/[^\s<]+/);
                    const url = match?.[0].replace(/[.,;:!?)}\]]+$/, "");
                    if (url) {
                        pushText(text.slice(start, index));
                        content.push({ type: "url", url });
                        index += url.length;
                        start = index;
                        continue;
                    }
                }
                if (text[index] === "{" || text[index] === "[") {
                    const end = jsonEnd(index);
                    if (end) {
                        try {
                            const json = JSON.parse(text.slice(index, end));
                            pushText(text.slice(start, index));
                            content.push({ type: "json", json });
                            index = end;
                            start = index;
                            continue;
                        } catch {}
                    }
                }
                index++;
            }
            pushText(text.slice(start));
            return content;
        }

        async function uploadAttachment(attachment) {
            if (attachment.artifactId) return;
            // Some text extensions have an empty or application/* browser MIME type.
            const file = attachment.kind === "text" && !attachment.file.type.startsWith("text/")
                ? new File([attachment.file], attachment.file.name, { type: "text/plain" })
                : attachment.file;
            const body = new FormData();
            body.append("files", file);
            const response = await fetch("/artifacts", { method: "POST", body });
            if (!response.ok) {
                const detail = (await response.text()).replace(/\s+/g, " ").trim().slice(0, 200);
                throw new Error(`Upload failed for ${file.name} (${response.status}). ${detail || "Try again."}`);
            }
            const artifacts = await response.json();
            if (!Array.isArray(artifacts) || artifacts.length !== 1 || typeof artifacts[0]?.id !== "string" || !artifacts[0].id) {
                throw new Error(`Invalid upload response for ${file.name}. Try again.`);
            }
            attachment.artifactId = artifacts[0].id;
        }

        async function submitChatMessage() {

            if (chatInput.readOnly) {
                return;
            }

            const state = window.Alpine.$data(orbScene);
            const text = state.draft.trim();

            if (!text && pendingAttachments.length === 0) {
                setChatError("Enter a message or attach a file.");
                chatInput.focus();
                return;
            }

            setChatError("");
            const restoreFocus = document.activeElement === chatSend;
            chatInput.readOnly = true;
            chatSend.disabled = true;
            chatAttach.disabled = true;
            chatFiles.disabled = true;
            renderAttachments();

            try {
                const uploads = pendingAttachments.filter((attachment) => !attachment.artifactId);
                for (const [index, attachment] of uploads.entries()) {
                    await uploadAttachment(attachment);
                }
                const parts = parseContent(text);
                parts.push(...pendingAttachments.map(({ artifactId }) => ({ type: "artifact", artifact_id: artifactId })));
                const content = parts.length === 1 && parts[0].type === "text" ? parts[0].text : parts;
                const path = chatId ? `/chats/${encodeURIComponent(chatId)}/messages` : "/chats/messages";
                setAwaitingResponse(true);
                const response = await fetch(path, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ role: "user", content }),
                });

                if (!response.ok) {
                    throw new Error("Message submission failed");
                }

                const message = await response.json();

                if (typeof message.chat_id !== "string" || !message.chat_id) {
                    throw new Error("Response is missing the chat ID");
                }

                const firstMessage = !chatId;
                chatId = message.chat_id;
                state.draft = "";
                requestAnimationFrame(resizeChatInput);
                pendingAttachments.length = 0;
                renderAttachments();
                mergeChatMessage(message);
                if (activity) activity.messageId = message.id;
                renderChatTranscript();

                if (firstMessage) {
                    window.history.replaceState({}, "", `/web/chat/${encodeURIComponent(chatId)}`);
                    void loadChatList();
                    void loadChatHistory();
                }
            } catch (error) {
                setAwaitingResponse(false, true);
                setChatError(error.message?.startsWith("Upload") || error.message?.startsWith("Invalid upload")
                    ? error.message : "Send failed. Try again.");
            } finally {
                chatInput.readOnly = false;
                chatSend.disabled = false;
                chatAttach.disabled = false;
                chatFiles.disabled = false;
                renderAttachments();

                if (restoreFocus && isChatOpen() && document.activeElement === document.body) {
                    chatInput.focus();
                }
            }
        }

        return {
            openAction, renderChatTranscript, submitChatMessage, handleChatFiles, setChatError,
            clearAwaitingResponse: () => setAwaitingResponse(false), closeDrawer, dismissOverlayDrawer,
        };
    }

    connectedCallback() {
        const content = document.getElementById("chat-template").content.cloneNode(true);
        customElements.upgrade(content);
        this.replaceChildren(content);
        this.listeners = new AbortController();
        this.controller = this.createView(this, this.listeners.signal);
    }

    disconnectedCallback() {
        this.listeners?.abort();
    }
});
