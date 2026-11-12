const orbScene = document.querySelector(".orb-scene");
const orbHub = document.querySelector("#orb-hub");
const orbTrigger = document.querySelector("#orb-trigger");
const chatAction = document.querySelector("#chat-action");
const memoryAction = document.querySelector("#memory-action");
const hubBack = document.querySelector("#hub-back");
const actionPanel = document.querySelector("#action-panel");
const actionTitle = document.querySelector("#action-title");
const actionBack = document.querySelector("#action-back");
const chatForm = document.querySelector("#chat-form");
const chatInput = document.querySelector("#chat-input");
const chatSend = document.querySelector("#chat-send");
const chatAttach = document.querySelector("#chat-attach");
const chatFiles = document.querySelector("#chat-files");
const chatAttachments = document.querySelector("#chat-attachments");
const pendingAttachments = [];
// Leave room for multipart headers within the gateway's 10 MiB request limit.
const maxAttachmentSize = 10 * 1024 * 1024 - 64 * 1024;
const textExtensions = /\.(txt|md|csv|json|log|ya?ml)$/i;
const attachmentPaths = {
    image: "M3 3h18v18H3z M3 17l6-6 5 5 3-3 4 4 M16 7h.01",
    audio: "M9 18V5l11-2v13 M9 8l11-2 M9 18c0 2-6 3-6 0s6-2 6 0 M20 16c0 2-6 3-6 0s6-2 6 0",
    text: "M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8 M8 16h8",
};
const chatError = document.querySelector("#chat-error");
const chatStatus = document.querySelector("#chat-status");
const chatTranscript = document.querySelector("#chat-transcript");
const chatHistoryStatus = document.querySelector("#chat-history-status");
const chatScroll = chatTranscript.querySelector(".chat-scroll");
const chatMessageList = document.querySelector("#chat-messages");
const chatMessages = new Map();
const signalDecoder = new TextDecoder();
const historyLimit = 100;

let chatId;
let activeAction;
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
    return orbScene.dataset.hud === "panel" && activeAction === chatAction;
}

function setHudView(view) {
    if (orbScene.dataset.hud === "memory" && view !== "memory") {
        memoryView.close();
    }

    orbScene.dataset.hud = view;
    orbHub.inert = view !== "hub";
    actionPanel.inert = view !== "panel";
    orbTrigger.inert = view === "memory";
    orbTrigger.setAttribute("aria-expanded", String(view === "hub"));
    renderChatTranscript();

    if (view === "hub") {
        void fetch("/memories?limit=5")
            .then((response) => {
                if (!response.ok) throw new Error("Memory request failed");
                return response.json();
            })
            .then((page) => {
                memoryAction.disabled = !page.data?.length;
            })
            .catch(() => {
                memoryAction.disabled = true;
            });
    }
}

function closeHub() {
    setHudView("idle");
    orbTrigger.focus();
}

function returnToHub() {
    setHudView("hub");
    activeAction.focus();
}

function openAction(action) {
    activeAction = action;

    if (action === memoryAction) {
        setHudView("memory");
        memoryView.open();
        return;
    }

    actionTitle.textContent = action.textContent.trim();
    const isChat = action === chatAction;
    actionPanel.classList.toggle("chat-panel", isChat);
    chatForm.hidden = !isChat;
    setHudView("panel");

    if (isChat) {
        if (!chatSend.disabled) {
            setChatStatus("");
        }

        chatInput.focus();
        void loadChatHistory();
    } else {
        actionBack.focus();
    }
}

orbTrigger.addEventListener("click", () => {
    if (orbScene.dataset.hud === "idle") {
        setHudView("hub");
        chatAction.focus();
    } else if (orbScene.dataset.hud === "panel") {
        returnToHub();
    }
});

for (const action of orbHub.querySelectorAll(".hub-action")) {
    action.addEventListener("click", () => openAction(action));
}

hubBack.addEventListener("click", closeHub);
actionBack.addEventListener("click", returnToHub);
document.querySelector("#memory-back").addEventListener("click", returnToHub);

document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || event.isComposing) {
        return;
    }

    if (orbScene.dataset.hud === "memory") {
        event.preventDefault();

        if (!memoryView.dismissDetails()) {
            returnToHub();
        }
    } else if (orbScene.dataset.hud === "panel") {
        event.preventDefault();
        returnToHub();
    } else if (orbScene.dataset.hud === "hub") {
        event.preventDefault();
        closeHub();
    }
});

function messageText(content) {
    if (typeof content === "string") {
        return content;
    }

    if (Array.isArray(content)) {
        return content.map(messageText).join("\n");
    }

    return content?.text ?? content?.url ?? (content?.artifact_id ? "Attachment" : "Message content");
}

function messageOrder(message) {
    const timestamp = message.created_at.replace(/(?:\.(\d+))?Z$/, (_, fraction = "") => `.${fraction.padEnd(9, "0")}Z`);
    return `${timestamp} ${message.id}`;
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
    chatTranscript.hidden = !isChatOpen() || chatMessages.size === 0;

    if (chatTranscript.hidden) {
        return;
    }

    const scrollTop = chatScroll.scrollTop;
    const stickToEnd = chatScroll.scrollHeight - chatScroll.clientHeight - scrollTop < 16;
    const messages = [...chatMessages.values()].sort((a, b) => messageOrder(a).localeCompare(messageOrder(b)));
    const rows = document.createDocumentFragment();

    for (const message of messages) {
        const row = document.createElement("li");
        const role = document.createElement("span");
        const text = document.createElement("p");
        role.className = "chat-message-role";
        role.textContent = message.role === "user" ? "You" : message.role;
        text.className = "chat-message-text";
        text.textContent = messageText(message.content);
        row.append(role, text);
        rows.append(row);
    }

    chatMessageList.replaceChildren(rows);
    chatHistoryStatus.textContent = historyLoading ? "Loading history…"
        : historyError ? "Unable to load chat history." : "";
    chatScroll.scrollTop = stickToEnd ? chatScroll.scrollHeight : scrollTop;
}

async function loadChatHistory() {
    if (!chatId || historyLoading) {
        return;
    }

    historyLoading = true;
    historyError = false;
    chatHistoryStatus.textContent = "Loading history…";

    try {
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
        if (memoryView.handleSignal(signal)) {
            memoryAction.disabled = !memoryView.hasMemories();
        }

        if (mergeChatMessage(signal?.content)) {
            renderChatTranscript();
        }
    } catch {
        return;
    }
});

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

chatAttach.addEventListener("click", () => chatFiles.click());
chatFiles.addEventListener("change", () => {
    if (chatSend.disabled) return;
    const errors = [];
    for (const file of chatFiles.files) {
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
});

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

chatInput.addEventListener("input", () => {
    setChatError("");
});

chatForm.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (chatSend.disabled) {
        return;
    }

    const text = chatInput.value.trim();

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
        chatInput.value = "";
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
});
