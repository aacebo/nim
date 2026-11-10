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

            const messages = await response.json();

            for (const message of messages) {
                mergeChatMessage(message);
            }

            const cursor = response.headers.get("x-next-cursor");

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
        }
    } catch {
        return;
    }
});

chatInput.addEventListener("input", () => {
    chatError.textContent = "";
});

chatForm.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (chatSend.disabled) {
        return;
    }

    const text = chatInput.value.trim();

    if (!text) {
        chatError.textContent = "Enter a message.";
        chatInput.focus();
        return;
    }

    chatError.textContent = "";
    const restoreFocus = document.activeElement === chatSend;
    chatInput.readOnly = true;
    chatSend.disabled = true;
    setChatStatus("Sending…");

    try {
        const path = chatId ? `/chats/${encodeURIComponent(chatId)}/messages` : "/chats/messages";
        const response = await fetch(path, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ role: "user", content: { text } }),
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
        mergeChatMessage(message);
        renderChatTranscript();

        if (firstMessage) {
            void loadChatHistory();
        }

        setChatStatus("Sent", true);
    } catch {
        setChatStatus("");
        chatError.textContent = "Send failed. Try again.";
    } finally {
        chatInput.readOnly = false;
        chatSend.disabled = false;

        if (restoreFocus && isChatOpen() && document.activeElement === document.body) {
            chatInput.focus();
        }
    }
});
