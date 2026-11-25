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
            if (scroll.scrollTop < 16) this.dispatchEvent(new CustomEvent("nim-chat-history", { bubbles: true }));
        }, true);
        this.resizeObserver = new ResizeObserver(() => {
            const chatTranscript = this.querySelector("#chat-transcript");
            const chatScroll = this.querySelector(".chat-scroll");
            this.fitScrollViewport();
            if (!chatTranscript || !chatScroll || chatTranscript.hidden || !this.followLatest) return;
            chatScroll.scrollTop = chatScroll.scrollHeight;
            this.scrollPosition = chatScroll.scrollTop;
            chatScroll.classList.toggle("has-earlier-messages", chatScroll.scrollTop > 0);
        });
    }

    connectedCallback() {
        const chatTranscript = this.querySelector("#chat-transcript");
        const chatMessageList = this.querySelector("#chat-messages");
        if (chatTranscript) this.resizeObserver.observe(chatTranscript);
        if (chatMessageList) this.resizeObserver.observe(chatMessageList);
    }

    disconnectedCallback() {
        this.resizeObserver.disconnect();
    }

    fitScrollViewport() {
        const chatTranscript = this.querySelector("#chat-transcript");
        const chatScroll = this.querySelector(".chat-scroll");
        const chatHistoryStatus = this.querySelector("#chat-history-status");
        if (!chatTranscript || !chatScroll || chatTranscript.hidden) return;
        const height = Math.max(0, chatTranscript.getBoundingClientRect().height - chatHistoryStatus.getBoundingClientRect().height);
        const value = `${height}px`;
        if (chatScroll.style.height !== value) chatScroll.style.height = value;
    }

    messageText(content) {
        if (typeof content === "string") return content;
        if (Array.isArray(content)) return content.map((item) => this.messageText(item)).filter(Boolean).join("");
        switch (content?.type) {
            case "text": return content.text;
            case "url": return content.url;
            case "json": return JSON.stringify(content.json);
            case "thinking": return "";
            case "artifact": return "";
            default: return "Message content";
        }
    }

    renderMarkdown(content) {
        const renderer = new marked.Renderer();
        renderer.html = () => "";
        return DOMPurify.sanitize(marked.parse(content, { gfm: true, renderer }));
    }

    messageArtifacts(content) {
        if (Array.isArray(content)) return content.flatMap((item) => this.messageArtifacts(item));
        return content?.type === "artifact" ? [{
            id: content.id,
            name: content.name,
            size: content.size,
            media_type: content.media_type,
        }] : [];
    }

    messageOrder(message) {
        const timestamp = message.created_at.replace(/(?:\.(\d+))?Z$/, (_, fraction = "") => `.${fraction.padEnd(9, "0")}Z`);
        return `${timestamp} ${message.id}`;
    }

    messageTimestamp(value) {
        const date = new Date(value);
        if (Number.isNaN(date.valueOf())) return value;
        const today = new Date();
        const isToday = date.getFullYear() === today.getFullYear()
            && date.getMonth() === today.getMonth()
            && date.getDate() === today.getDate();
        return date.toLocaleString([], isToday
            ? { hour: "numeric", minute: "2-digit" }
            : { month: "short", day: "numeric" });
    }

    activityRow(activity) {
        const row = document.createElement("li");
        const details = document.createElement("details");
        const summary = document.createElement("summary");
        const label = document.createElement("span");

        row.className = "chat-activity";
        row.classList.toggle("is-active", activity.active);
        details.open = activity.expanded;
        summary.addEventListener("click", () => {
            setTimeout(() => this.dispatchEvent(new CustomEvent("nim-chat-activity-toggle", {
                bubbles: true,
                detail: { expanded: details.open },
            })));
        });
        label.className = "chat-activity-label";
        label.textContent = activity.active ? activity.elapsed ? `Working for ${activity.elapsed}` : "Working…"
            : `Worked for ${activity.elapsed}`;
        summary.append(label);
        details.append(summary);

        if (activity.active || activity.updates.length) {
            const current = document.createElement("p");
            current.className = "chat-activity-current";
            current.setAttribute("aria-live", "polite");
            current.textContent = activity.updates.at(-1)?.text || "Working…";
            details.append(current);
        }

        if (activity.updates.length > 1) {
            const updates = document.createElement("ol");
            updates.className = "chat-activity-updates";
            for (const update of activity.updates.slice(0, -1)) {
                const item = document.createElement("li");
                item.textContent = update.text;
                updates.append(item);
            }
            details.append(updates);
        }

        row.append(details);

        if (activity.active) {
            const bubble = document.createElement("div");
            const first = document.createElement("span");
            const second = document.createElement("span");
            bubble.className = "chat-message-bubble chat-message-loading";
            bubble.style.setProperty("--loading-elapsed", `${activity.startedAt - Date.now()}ms`);
            bubble.setAttribute("role", "status");
            bubble.setAttribute("aria-label", "Generating response");
            bubble.append(first, second);
            row.append(bubble);
        }

        return row;
    }

    render({ open, chatMessages, historyLoading, historyError, activity }) {
        const chatTranscript = this.querySelector("#chat-transcript");
        const chatScroll = this.querySelector(".chat-scroll");
        const chatMessageList = this.querySelector("#chat-messages");
        const chatHistoryStatus = this.querySelector("#chat-history-status");
        const wasHidden = chatTranscript.hidden;

        if (!wasHidden) {
            this.scrollPosition = chatScroll.scrollTop;
            this.followLatest = chatScroll.scrollHeight - chatScroll.clientHeight - chatScroll.scrollTop < 16;
        }

        chatTranscript.hidden = !open || chatMessages.size === 0 && !activity;

        if (chatTranscript.hidden) {
            return;
        }

        const messages = [...chatMessages.values()].sort((a, b) => this.messageOrder(a).localeCompare(this.messageOrder(b)));
        const rows = document.createDocumentFragment();

        for (const message of messages) {
            const row = document.createElement("li");
            const body = document.createElement("div");
            const bubble = document.createElement("div");
            const metadata = document.createElement("div");
            const timestamp = document.createElement("time");
            const actions = document.createElement("div");
            const content = this.messageText(message.content);
            const text = document.createElement("div");
            const isUser = message.role === "user";
            row.className = isUser ? "chat-message-user" : "chat-message-agent";
            body.className = "chat-message-body";
            bubble.className = "chat-message-bubble";
            metadata.className = "chat-message-footer";
            timestamp.className = "chat-message-timestamp";
            timestamp.dateTime = message.created_at;
            timestamp.textContent = this.messageTimestamp(message.created_at);
            actions.className = "chat-message-actions";
            const copy = document.createElement("button");
            copy.type = "button";
            copy.setAttribute("aria-label", "Copy message");
            copy.title = "Copy message";
            copy.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="8" y="8" width="11" height="11" rx="2"></rect><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"></path></svg>';
            copy.addEventListener("click", () => { void navigator.clipboard?.writeText(content)?.catch(() => {}); });
            const inspect = document.createElement("button");
            inspect.type = "button";
            inspect.setAttribute("aria-label", "View message details");
            inspect.title = "View message details";
            inspect.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="10.8" cy="10.8" r="5.8"></circle><path d="m15.2 15.2 4.3 4.3"></path></svg>';
            inspect.addEventListener("click", () => this.dispatchEvent(new CustomEvent("nim-message-inspect", { bubbles: true, detail: { id: message.id, trigger: inspect } })));
            actions.append(copy, inspect);
            metadata.append(timestamp, actions);
            text.className = "chat-message-text";
            text.innerHTML = this.renderMarkdown(content);
            for (const link of text.querySelectorAll("a")) {
                link.target = "_blank";
                link.rel = "noopener noreferrer";
            }
            if (content) bubble.append(text);
            const attachments = this.messageArtifacts(message.content);
            if (attachments.length && window.nimArtifact) body.append(window.nimArtifact.list(attachments));
            if (content) body.append(bubble);
            body.append(metadata);
            row.append(body);
            rows.append(row);
            if (activity?.messageId === message.id) rows.append(this.activityRow(activity));
        }

        if (activity && !activity.messageId) rows.append(this.activityRow(activity));

        chatMessageList.replaceChildren(rows);
        chatHistoryStatus.textContent = historyLoading ? "Loading history…"
            : historyError ? "Unable to load chat history." : "";
        const restoreScroll = () => {
            this.fitScrollViewport();
            chatScroll.scrollTop = this.followLatest ? chatScroll.scrollHeight : this.scrollPosition;
            this.scrollPosition = chatScroll.scrollTop;
            chatScroll.classList.toggle("has-earlier-messages", chatScroll.scrollTop > 0);
        };
        restoreScroll();
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                if (!chatTranscript.hidden) restoreScroll();
            });
        });
    }
});
