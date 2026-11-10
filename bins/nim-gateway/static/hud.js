const orbScene = document.querySelector(".orb-scene");
const animusHub = document.querySelector("#animus-hub");
const orbTrigger = document.querySelector("#orb-trigger");
const chatAction = document.querySelector("#chat-action");
const hubBack = document.querySelector("#hub-back");
const actionPanel = document.querySelector("#action-panel");
const actionTitle = document.querySelector("#action-title");
const actionBack = document.querySelector("#action-back");
const chatForm = document.querySelector("#chat-form");
const chatInput = document.querySelector("#chat-input");
const chatSend = document.querySelector("#chat-send");
const chatError = document.querySelector("#chat-error");
const chatStatus = document.querySelector("#chat-status");

let chatId;
let activeAction;

function setHudView(view) {
    orbScene.dataset.hud = view;
    animusHub.inert = view !== "hub";
    actionPanel.inert = view !== "panel";
    orbTrigger.setAttribute("aria-expanded", String(view === "hub"));
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
    actionTitle.textContent = action.textContent.trim();
    const isChat = action === chatAction;
    actionPanel.classList.toggle("chat-panel", isChat);
    chatForm.hidden = !isChat;
    setHudView("panel");

    if (isChat) {
        if (!chatSend.disabled) {
            chatStatus.textContent = "";
        }

        chatInput.focus();
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

for (const action of animusHub.querySelectorAll(".hub-action")) {
    action.addEventListener("click", () => openAction(action));
}

hubBack.addEventListener("click", closeHub);
actionBack.addEventListener("click", returnToHub);

document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || event.isComposing) {
        return;
    }

    if (orbScene.dataset.hud === "panel") {
        event.preventDefault();
        returnToHub();
    } else if (orbScene.dataset.hud === "hub") {
        event.preventDefault();
        closeHub();
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
    chatInput.readOnly = true;
    chatSend.disabled = true;
    chatStatus.textContent = "Sending…";

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

        chatId = message.chat_id;
        chatInput.value = "";

        if (orbScene.dataset.hud === "panel" && activeAction === chatAction) {
            closeHub();
        }

        chatStatus.textContent = "Sent";
    } catch {
        chatStatus.textContent = orbScene.dataset.hud === "panel" && activeAction === chatAction ? "" : "Send failed";
        chatError.textContent = "Send failed. Try again.";
    } finally {
        chatInput.readOnly = false;
        chatSend.disabled = false;
    }
});
