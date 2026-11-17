customElements.define("nim-chat", class extends HTMLElement {
    createView(host, signal) {
        const orbScene = document.querySelector(".orb-scene");
        const actionPanel = host.querySelector("#action-panel");
        const actionTitle = host.querySelector("#action-title");
        const actionBack = host.querySelector("#action-back");
        const chatForm = host.querySelector("#chat-form");
        const chatInput = host.querySelector("#chat-input");
        const chatSend = host.querySelector("#chat-send");
        const chatAttach = host.querySelector("#chat-attach");
        const chatFiles = host.querySelector("#chat-files");
        const chatAttachments = host.querySelector("#chat-attachments");
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
        const chatStatus = host.querySelector("#chat-status");
        const chatHistoryStatus = host.querySelector("#chat-history-status");
        const chatMessages = new Map();
        const signalDecoder = new TextDecoder();
        const historyLimit = 100;

        let chatId;
        let historyLoading = false;
        let historyError = false;
        let chatStatusTimer;
        let chatErrorTimer;

        function setChatStatus(text, dismiss = false) {
            clearTimeout(chatStatusTimer);
            chatStatus.classList.remove("is-fading");
            chatStatus.textContent = text;

            if (dismiss) {
                chatStatusTimer = setTimeout(() => {
                    chatStatus.classList.add("is-fading");
                    chatStatusTimer = setTimeout(() => {
                        chatStatus.textContent = "";
                    }, 300);
                }, 2200);
            }
        }

        function setChatError(text) {
            clearTimeout(chatErrorTimer);
            chatError.classList.remove("is-fading");
            chatError.textContent = text;
            if (!text) return;
            setChatStatus("");
            chatErrorTimer = setTimeout(() => {
                chatError.classList.add("is-fading");
                chatErrorTimer = setTimeout(() => {
                    chatError.textContent = "";
                }, 300);
            }, 2200);
        }

        function isChatOpen() {
            const state = Alpine.$data(orbScene);
            return state.hud === "panel" && state.showChatForm;
        }

        function openAction(isChat, title) {
            actionTitle.textContent = title;
            actionPanel.classList.toggle("chat-panel", isChat);
            renderChatTranscript();
            if (isChat) {
                if (!chatSend.disabled) setChatStatus("");
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
            } else {
                Alpine.nextTick(() => actionBack.focus());
            }
        }

        async function loadChatArtifacts(id) {
            const query = new URLSearchParams({ limit: "100" });
            const cursors = new Set();
            while (true) {
                const response = await fetch(`/chats/${encodeURIComponent(id)}/artifacts?${query}`);
                if (!response.ok) throw new Error("Unable to load attachments");
                const page = await response.json();
                host.closest("#nim-app").querySelector("nim-artifacts").controller.remember(page.data);
                if (!page.next) break;
                if (cursors.has(page.next)) throw new Error("Attachment cursor did not advance");
                cursors.add(page.next);
                query.set("cursor", page.next);
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
                open: isChatOpen(), chatMessages, historyLoading, historyError,
                artifactView: host.closest("#nim-app").querySelector("nim-artifacts").controller,
            });
        }

        async function loadChatHistory() {
            if (!chatId || historyLoading) {
                return;
            }

            historyLoading = true;
            historyError = false;
            chatHistoryStatus.textContent = "Loading history…";

            try {
                await loadChatArtifacts(chatId).catch(() => { historyError = true; });
                const query = new URLSearchParams({ limit: historyLimit });

                while (true) {
                    const response = await fetch(`/chats/${encodeURIComponent(chatId)}/messages?${query}`);

                    if (!response.ok) {
                        throw new Error("Chat history request failed");
                    }

                    const page = await response.json();

                    for (const message of page.data) {
                        mergeChatMessage(message);
                    }

                    const cursor = page.next;

                    if (!cursor) {
                        break;
                    }

                    query.set("cursor", cursor);
                }
            } catch {
                historyError = true;
            } finally {
                historyLoading = false;
                renderChatTranscript();
            }
        }

        socket.binaryType = "arraybuffer";
        socket.addEventListener("message", (event) => {
            try {
                const payload = typeof event.data === "string" ? event.data : signalDecoder.decode(event.data);
                const signal = JSON.parse(payload);
                if (mergeChatMessage(signal?.content)) {
                    renderChatTranscript();
                    if (host.querySelector("nim-chat-transcript").messageArtifacts(signal.content.content, host.closest("#nim-app").querySelector("nim-artifacts").controller).length) {
                        void loadChatArtifacts(chatId).then(renderChatTranscript).catch(() => {});
                    }
                }
            } catch {
                return;
            }
        }, { signal });

        function renderAttachments() {
            chatAttachments.hidden = pendingAttachments.length === 0;
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
            host.closest("#nim-app").querySelector("nim-artifacts").controller.remember(artifacts);
            attachment.artifactId = artifacts[0].id;
        }

        async function submitChatMessage() {

            if (chatSend.disabled) {
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
                    setChatStatus(`Uploading ${index + 1}/${uploads.length}…`);
                    await uploadAttachment(attachment);
                }
                const content = [];
                if (text) content.push({ text });
                content.push(...pendingAttachments.map(({ artifactId }) => ({ artifact_id: artifactId })));
                setChatStatus("Sending…");
                const path = chatId ? `/chats/${encodeURIComponent(chatId)}/messages` : "/chats/messages";
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
                pendingAttachments.length = 0;
                renderAttachments();
                mergeChatMessage(message);
                renderChatTranscript();

                if (firstMessage) {
                    void loadChatHistory();
                }

                setChatStatus("Sent", true);
            } catch (error) {
                setChatStatus("");
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

        return { openAction, renderChatTranscript, submitChatMessage, handleChatFiles, setChatError };
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