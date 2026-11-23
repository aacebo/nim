customElements.define("nim-chat-transcript", class extends HTMLElement {
    constructor() {
        super();
        this.followLatest = true;
        this.scrollPosition = 0;

        this.addEventListener("scroll", (event) => {
            const scroll = event.target;
            if (!scroll.classList.contains("chat-scroll") || this.querySelector("#chat-transcript").hidden) return;

            this.scrollPosition = scroll.scrollTop;
            this.followLatest = scroll.scrollHeight - scroll.clientHeight - scroll.scrollTop < 16;
            scroll.classList.toggle("has-earlier-messages", scroll.scrollTop > 0);
        }, true);
    }

    messageText(content) {
        if (typeof content === "string") return content;
        if (Array.isArray(content)) return content.map((item) => this.messageText(item)).filter(Boolean).join("\n");
        switch (content?.type) {
            case "text": return content.text;
            case "url": return content.url;
            case "json": return JSON.stringify(content.json);
            case "artifact": return "";
            default: return "Message content";
        }
    }

    renderMarkdown(content) {
        const renderer = new marked.Renderer();
        renderer.html = () => "";
        return DOMPurify.sanitize(marked.parse(content, { gfm: true, renderer }));
    }

    messageArtifacts(content, artifacts) {
        if (Array.isArray(content)) return content.flatMap((item) => this.messageArtifacts(item, artifacts));
        return content?.type === "artifact" ? [artifacts?.get(content.artifact_id) ?? { id: content.artifact_id }] : [];
    }

    messageOrder(message) {
        const timestamp = message.created_at.replace(/(?:\.(\d+))?Z$/, (_, fraction = "") => `.${fraction.padEnd(9, "0")}Z`);
        return `${timestamp} ${message.id}`;
    }

    messageTimestamp(value) {
        const date = new Date(value);
        return Number.isNaN(date.valueOf()) ? value : date.toLocaleString([], {
            month: "short", day: "numeric", hour: "numeric", minute: "2-digit",
        });
    }

    render({ open, chatMessages, historyLoading, historyError, artifacts }) {
        const chatTranscript = this.querySelector("#chat-transcript");
        const chatScroll = this.querySelector(".chat-scroll");
        const chatMessageList = this.querySelector("#chat-messages");
        const chatHistoryStatus = this.querySelector("#chat-history-status");
        const wasHidden = chatTranscript.hidden;

        if (!wasHidden) {
            this.scrollPosition = chatScroll.scrollTop;
            this.followLatest = chatScroll.scrollHeight - chatScroll.clientHeight - chatScroll.scrollTop < 16;
        }

        chatTranscript.hidden = !open || chatMessages.size === 0;

        if (chatTranscript.hidden) {
            return;
        }

        const messages = [...chatMessages.values()].sort((a, b) => this.messageOrder(a).localeCompare(this.messageOrder(b)));
        const rows = document.createDocumentFragment();

        for (const message of messages) {
            const row = document.createElement("li");
            const role = document.createElement("span");
            const header = document.createElement("div");
            const timestamp = document.createElement("time");
            const actions = document.createElement("div");
            const content = this.messageText(message.content);
            const text = document.createElement("div");
            const isUser = message.role === "user";
            row.className = isUser ? "chat-message-user" : "chat-message-agent";
            role.className = "chat-message-role";
            role.textContent = isUser ? "You" : message.role;
            timestamp.className = "chat-message-timestamp";
            timestamp.dateTime = message.created_at;
            timestamp.textContent = this.messageTimestamp(message.created_at);
            actions.className = "chat-message-actions";
            const action = document.createElement("button");
            action.type = "button";
            action.setAttribute("aria-label", "Inspect message");
            action.title = "Inspect message";
            action.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="10.8" cy="10.8" r="5.8"></circle><path d="m15.2 15.2 4.3 4.3"></path></svg>';
            action.addEventListener("click", () => this.dispatchEvent(new CustomEvent("nim-message-inspect", { bubbles: true, detail: { id: message.id, trigger: action } })));
            actions.append(action);
            header.className = "chat-message-header";
            header.append(role, timestamp, actions);
            text.className = "chat-message-text";
            text.innerHTML = this.renderMarkdown(content);
            for (const link of text.querySelectorAll("a")) {
                link.target = "_blank";
                link.rel = "noopener noreferrer";
            }
            row.append(header);
            if (content) row.append(text);
            const attachments = this.messageArtifacts(message.content, artifacts);
            if (attachments.length && window.nimArtifact) row.append(window.nimArtifact.list(attachments));
            rows.append(row);
        }

        chatMessageList.replaceChildren(rows);
        chatHistoryStatus.textContent = historyLoading ? "Loading history…"
            : historyError ? "Unable to load chat history." : "";
        chatScroll.scrollTop = this.followLatest ? chatScroll.scrollHeight : this.scrollPosition;
        this.scrollPosition = chatScroll.scrollTop;
        chatScroll.classList.toggle("has-earlier-messages", chatScroll.scrollTop > 0);
    }
});
