customElements.define("nim-message", class extends HTMLElement {
    connectedCallback() {
        const template = document.getElementById("message-template");
        const { message, annotations } = template.dataset;
        this.message = JSON.parse(message);
        this.annotations = JSON.parse(annotations);
        this.artifacts = this.message.artifacts || [];
        this.append(template.content.cloneNode(true));
        this.dialog = this.querySelector("dialog");
        this.querySelector("[data-close]")?.addEventListener("click", () => this.dialog.close());
        this.querySelectorAll("[data-tab]").forEach((button) => { button.onclick = () => this.render(button.dataset.tab); });
    }

    formatDate(value) {
        return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(value));
    }

    label(value) {
        return String(value).replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
    }

    percent(value) {
        return `${Math.round((value || 0) * 100)}%`;
    }

    confidenceClass(value) {
        const percent = Math.round((value || 0) * 100);
        return percent < 50 ? "is-low" : percent < 80 ? "is-medium" : "is-high";
    }

    messageText(content) {
        if (typeof content === "string") return content;
        if (Array.isArray(content)) return content.map((item) => this.messageText(item)).filter(Boolean).join("");
        switch (content?.type) {
            case "text": return content.text;
            case "url": return content.url;
            case "json": return JSON.stringify(content.json, null, 2);
            default: return "";
        }
    }

    renderMarkdown(content) {
        const renderer = new marked.Renderer();
        renderer.html = () => "";
        return DOMPurify.sanitize(marked.parse(content, { gfm: true, renderer }));
    }

    async copyPayload(button) {
        await navigator.clipboard.writeText(JSON.stringify(this.message, null, 2));
        const icon = button.querySelector("svg");
        if (!button || !icon) return;
        clearTimeout(this.copyTimer);
        icon.innerHTML = '<path d="m5 12 4.2 4.2L19 6.5"></path>';
        button.classList.add("is-copied");
        this.copyTimer = setTimeout(() => {
            icon.innerHTML = '<rect x="8" y="8" width="11" height="11" rx="2"></rect><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"></path>';
            button.classList.remove("is-copied");
        }, 2000);
    }

    render(tab) {
        const body = this.querySelector(".message-view-body");
        body.classList.toggle("is-content", tab === "content");
        body.classList.toggle("is-json", tab === "json");
        this.querySelectorAll("[data-tab]").forEach((button) => {
            button.ariaSelected = String(button.dataset.tab === tab);
            button.tabIndex = button.dataset.tab === tab ? 0 : -1;
        });
        body.replaceChildren(tab === "content" ? this.contentView() : tab === "json" ? this.jsonView() : this.detailsView());
    }

    contentView() {
        const section = document.createElement("section");
        section.className = "message-content";
        const thinking = Array.isArray(this.message.content)
            ? this.message.content.filter((part) => part?.type === "thinking" && part.thinking?.trim()).map((part) => part.thinking).join("\n\n")
            : "";
        const content = document.createElement("div");
        content.className = "message-content-markdown";
        if (thinking) {
            const details = document.createElement("details");
            const summary = document.createElement("summary");
            const reasoning = document.createElement("div");
            details.className = "message-content-thinking";
            summary.textContent = "Thinking";
            reasoning.innerHTML = this.renderMarkdown(thinking);
            details.append(summary, reasoning);
            content.append(details);
        }
        const answer = document.createElement("div");
        answer.innerHTML = this.renderMarkdown(this.messageText(this.message.content));
        content.append(answer);
        for (const link of content.querySelectorAll("a")) {
            link.target = "_blank";
            link.rel = "noopener noreferrer";
        }
        section.append(content);
        return section;
    }

    detailsView() {
        const fragment = document.createDocumentFragment();
        const summaryHeader = document.createElement("div");
        summaryHeader.className = "message-summary-header";
        const summary = document.createElement("section");
        summary.className = "message-summary";
        const title = document.createElement("h4");
        title.textContent = "Summary";
        summaryHeader.append(title);
        if (this.message.confidence > 0) {
            const confidence = document.createElement("strong");
            confidence.textContent = `${this.percent(this.message.confidence)} confidence`;
            summaryHeader.append(confidence);
        }
        const text = document.createElement("p");
        text.textContent = this.message.summary || "No summary available.";
        summary.append(text);
        const metadata = document.createElement("dl");
        metadata.className = "message-metadata";
        for (const [label, value] of [["Confidence", this.message.confidence]].filter(([, value]) => Number.isFinite(value))) {
            const row = document.createElement("div");
            const key = document.createElement("dt");
            const detail = document.createElement("dd");
            key.textContent = label;
            detail.textContent = this.percent(value);
            row.append(key, detail);
            metadata.append(row);
        }
        fragment.append(summaryHeader, summary, metadata, this.annotationView(), this.artifactView());
        return fragment;
    }

    sectionHeader(title, count, label) {
        const header = document.createElement("div");
        header.className = "message-section-header";
        const heading = document.createElement("h4");
        const total = document.createElement("span");
        heading.textContent = title;
        total.textContent = `${count} ${label}`;
        header.append(heading, total);
        return header;
    }

    artifactType(artifact) {
        const type = artifact.media_type?.split(";", 1)[0].trim().toLowerCase();
        if (type?.startsWith("image/")) return "image";
        if (type?.startsWith("audio/")) return "audio";
        if (type?.startsWith("video/")) return "video";
        const extension = artifact.name?.split(".").pop()?.toLowerCase();
        if (["avif", "gif", "jpeg", "jpg", "png", "svg", "webp"].includes(extension)) return "image";
        if (["aac", "flac", "m4a", "mp3", "ogg", "wav"].includes(extension)) return "audio";
        if (["mkv", "mov", "mp4", "webm"].includes(extension)) return "video";
        return "document";
    }

    artifactIcon(type) {
        if (type === "image") return '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3.5" y="4" width="17" height="16" rx="2"></rect><path d="m4 16 5-5 3.5 3.5 2.5-2.5 5 5M16.5 8.5h.01"></path></svg>';
        if (type === "audio") return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 18V6l10-2v12M9 9l10-2M9 18c0 2-5 3-5 0s5-2 5 0M19 16c0 2-5 3-5 0s5-2 5 0"></path></svg>';
        if (type === "video") return '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3.5" y="5" width="17" height="14" rx="2"></rect><path d="m10 9 5 3-5 3z"></path></svg>';
        return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 2h9l5 5v15H5zM14 2v6h5M8 12h8M8 16h8"></path></svg>';
    }

    async openArtifact(artifact, trigger) {
        await openArtifactView(artifact.id, trigger);
    }

    annotationView() {
        const section = document.createElement("section");
        section.className = "message-section";
        section.append(this.sectionHeader("Annotations", this.annotations.length, this.annotations.length === 1 ? "classification" : "classifications"));
        const table = document.createElement("div");
        table.className = "message-annotation-table";
        if (!this.annotations.length) {
            const empty = document.createElement("p");
            empty.className = "message-empty";
            empty.textContent = "No annotations linked to this message.";
            table.append(empty);
        }
        for (const annotation of this.annotations) {
            const row = document.createElement("div");
            row.className = "message-annotation-row";
            const line = document.createElement("div");
            const label = document.createElement("span");
            const text = document.createElement("span");
            const value = document.createElement("span");
            label.textContent = this.label(annotation.label);
            text.textContent = annotation.text;
            value.textContent = this.percent(annotation.confidence);
            line.append(label, text, value);
            const progress = document.createElement("div");
            const fill = document.createElement("span");
            fill.className = this.confidenceClass(annotation.confidence);
            fill.style.width = this.percent(annotation.confidence);
            progress.append(fill);
            row.append(line, progress);
            table.append(row);
        }
        section.append(table);
        return section;
    }

    artifactView() {
        const section = document.createElement("section");
        section.className = "message-section";
        section.append(this.sectionHeader("Artifacts", this.artifacts.length, "attached"));
        const table = document.createElement("div");
        table.className = "message-artifact-table";
        if (!this.artifacts.length) {
            const empty = document.createElement("p");
            empty.className = "message-empty";
            empty.textContent = "No artifacts linked to this message.";
            table.append(empty);
        }
        for (const artifact of this.artifacts) {
            const row = document.createElement("button");
            row.type = "button";
            row.className = "message-artifact-row";
            row.title = `View ${artifact.name || "artifact"}`;
            const icon = document.createElement("span");
            icon.className = "message-artifact-icon";
            const type = this.artifactType(artifact);
            icon.dataset.kind = type;
            icon.innerHTML = this.artifactIcon(type);
            const content = document.createElement("div");
            const name = document.createElement("strong");
            const detail = document.createElement("span");
            name.textContent = artifact.name;
            detail.textContent = [artifact.media_type, artifact.type === "resource" ? "Resource" : "Artifact"].filter(Boolean).join(" · ");
            content.append(name, detail);
            const open = document.createElement("span");
            open.className = "message-artifact-open";
            open.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M14 5h5v5M19 5l-8 8"></path><path d="M18 13v5a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"></path></svg>';
            row.append(icon, content, open);
            row.onclick = () => { void this.openArtifact(artifact, row); };
            table.append(row);
        }
        section.append(table);
        return section;
    }

    jsonView() {
        const section = document.createElement("section");
        section.className = "message-json";
        const heading = document.createElement("div");
        const title = document.createElement("h4");
        const controls = document.createElement("div");
        title.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 4H6a2 2 0 0 0-2 2v4M8 20H6a2 2 0 0 1-2-2v-4M16 4h2a2 2 0 0 1 2 2v4M16 20h2a2 2 0 0 0 2-2v-4"></path></svg>Message payload';
        for (const [label, open] of [["Expand all", true], ["Collapse all", false]]) {
            const button = document.createElement("button");
            button.type = "button";
            button.textContent = label;
            button.onclick = () => this.querySelectorAll(".message-json-tree details").forEach((detail) => { detail.open = open; });
            controls.append(button);
        }
        const copy = document.createElement("button");
        copy.type = "button";
        copy.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="8" y="8" width="11" height="11" rx="2"></rect><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"></path></svg><span>Copy</span>';
        copy.onclick = () => this.copyPayload(copy);
        controls.append(copy);
        heading.append(title, controls);
        const tree = document.createElement("div");
        tree.className = "message-json-tree";
        tree.tabIndex = 0;
        tree.addEventListener("wheel", (event) => {
            if (event.shiftKey || Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return;
            const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? tree.clientHeight : 1;
            const previous = tree.scrollTop;
            tree.scrollTop = Math.max(0, Math.min(tree.scrollTop + event.deltaY * unit, tree.scrollHeight - tree.clientHeight));
            if (tree.scrollTop !== previous) {
                event.preventDefault();
                event.stopPropagation();
            }
        }, { passive: false });
        tree.append(this.treeRoot(this.message));
        requestAnimationFrame(() => { tree.scrollTop = 0; });
        section.append(heading, tree);
        return section;
    }

    treeRoot(value) {
        const detail = document.createElement("details");
        detail.open = true;
        const summary = document.createElement("summary");
        summary.textContent = `{${Object.keys(value).length} properties}`;
        const children = document.createElement("div");
        children.className = "message-json-children";
        for (const [key, childValue] of Object.entries(value)) children.append(this.treeNode(key, childValue));
        detail.append(summary, children);
        return detail;
    }

    treeNode(key, value, open = false) {
        if (value !== null && typeof value === "object") {
            const detail = document.createElement("details");
            detail.open = open;
            const summary = document.createElement("summary");
            const name = document.createElement("span");
            const kind = document.createElement("span");
            name.textContent = `${key}:`;
            kind.textContent = Array.isArray(value) ? `Array(${value.length})` : `{${Object.keys(value).length} properties}`;
            summary.append(name, kind);
            const children = document.createElement("div");
            children.className = "message-json-children";
            for (const [childKey, childValue] of Object.entries(value)) children.append(this.treeNode(childKey, childValue));
            detail.append(summary, children);
            return detail;
        }
        const line = document.createElement("div");
        line.className = "message-json-value";
        const name = document.createElement("span");
        const content = document.createElement("span");
        name.textContent = `${key}:`;
        content.textContent = value === null ? "null" : typeof value === "string" ? `"${value}"` : String(value);
        content.dataset.type = value === null ? "null" : typeof value;
        line.append(name, content);
        return line;
    }

    open(trigger) {
        this.trigger = trigger;
        this.querySelector("[data-role]").textContent = this.label(this.message.role);
        this.querySelector("[data-id]").textContent = this.message.id;
        const timestamp = this.querySelector("time");
        timestamp.dateTime = this.message.created_at;
        timestamp.textContent = this.formatDate(this.message.created_at);
        this.render("content");
        this.dialog.showModal();
        this.dialog.addEventListener("close", () => this.trigger?.focus(), { once: true });
    }
});
