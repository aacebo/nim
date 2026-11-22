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
        return content?.text ?? content?.url ?? (content?.artifact_id ? "" : "Message content");
    }

    renderMarkdown(content) {
        const renderer = new marked.Renderer();
        renderer.html = () => "";
        return DOMPurify.sanitize(marked.parse(content, { gfm: true, renderer }));
    }

    messageArtifacts(content, artifactView) {
        if (Array.isArray(content)) return content.flatMap((item) => this.messageArtifacts(item, artifactView));
        return content?.artifact_id ? [artifactView?.get(content.artifact_id) ?? { id: content.artifact_id }] : [];
    }

    messageOrder(message) {
        const timestamp = message.created_at.replace(/(?:\.(\d+))?Z$/, (_, fraction = "") => `.${fraction.padEnd(9, "0")}Z`);
        return `${timestamp} ${message.id}`;
    }

    render({ open, chatMessages, historyLoading, historyError, artifactView }) {
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
            const content = this.messageText(message.content);
            const text = document.createElement("div");
            const isUser = message.role === "user";
            row.className = isUser ? "chat-message-user" : "chat-message-agent";
            role.className = "chat-message-role";
            role.textContent = isUser ? "You" : message.role;
            text.className = "chat-message-text";
            text.innerHTML = this.renderMarkdown(content);
            for (const link of text.querySelectorAll("a")) {
                link.target = "_blank";
                link.rel = "noopener noreferrer";
            }
            row.append(role);
            if (content) row.append(text);
            const artifacts = this.messageArtifacts(message.content, artifactView);
            if (artifacts.length && artifactView) row.append(artifactView.list(artifacts));
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
