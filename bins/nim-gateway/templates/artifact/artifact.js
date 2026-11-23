(() => {
    const blobs = new Map();
    const extensions = {
        avif: "image/avif", gif: "image/gif", jpeg: "image/jpeg", jpg: "image/jpeg", png: "image/png", svg: "image/svg+xml", webp: "image/webp",
        m4a: "audio/mp4", mp3: "audio/mpeg", ogg: "audio/ogg", wav: "audio/wav", mp4: "video/mp4", webm: "video/webm",
        csv: "text/csv", json: "application/json", log: "text/plain", md: "text/plain", txt: "text/plain", yaml: "text/plain", yml: "text/plain",
    };
    const paths = {
        image: "M3 3h18v18H3z M3 17l6-6 5 5 3-3 4 4 M16 7h.01",
        audio: "M9 18V5l11-2v13 M9 8l11-2 M9 18c0 2-6 3-6 0s6-2 6 0 M20 16c0 2-6 3-6 0s6-2 6 0",
        video: "M3 5h18v14H3z M10 9l5 3-5 3z",
        text: "M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8M8 16h8",
        file: "M5 2h9l5 5v15H5z M14 2v6h5",
    };

    const mime = (artifact) => {
        if (artifact.type === "text") return "text/plain";
        const declared = artifact.media_type?.split(";", 1)[0].trim().toLowerCase();
        return declared && declared !== "application/octet-stream"
            ? declared
            : extensions[artifact.name?.split(".").pop()?.toLowerCase()] || "application/octet-stream";
    };
    const kind = (artifact) => {
        const type = mime(artifact);
        if (type.startsWith("image/")) return "image";
        if (type.startsWith("audio/")) return "audio";
        if (type.startsWith("video/")) return "video";
        if (type.startsWith("text/") || ["application/json", "application/xml"].includes(type)) return "text";
        return "file";
    };
    const contentUrl = (artifact) => `/artifacts/${encodeURIComponent(artifact.id)}/content`;
    const blob = (artifact) => {
        const key = `${artifact.id}:${artifact.version ?? 0}`;
        if (!blobs.has(key)) {
            const request = fetch(contentUrl(artifact)).then(async (response) => {
                if (!response.ok) throw new Error(`Artifact content request failed (HTTP ${response.status}).`);
                const data = await response.blob();
                if (!data.size) throw new Error("The content endpoint returned an empty file.");
                return data.slice(0, data.size, mime(artifact));
            }).catch((error) => {
                blobs.delete(key);
                throw error;
            });
            blobs.set(key, request);
            if (blobs.size > 12) blobs.delete(blobs.keys().next().value);
        }
        return blobs.get(key);
    };
    const icon = (artifact) => {
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
        svg.setAttribute("viewBox", "0 0 24 24");
        svg.setAttribute("aria-hidden", "true");
        svg.classList.add("artifact-icon");
        path.setAttribute("d", paths[kind(artifact)]);
        svg.append(path);
        return svg;
    };
    const image = (artifact, className = "") => {
        const img = document.createElement("img");
        img.className = className;
        img.alt = artifact.name || "Artifact image";
        img.decoding = "async";
        void blob(artifact).then((data) => {
            const url = URL.createObjectURL(data);
            img.onload = () => URL.revokeObjectURL(url);
            img.onerror = () => URL.revokeObjectURL(url);
            img.src = url;
        }).catch(() => { img.remove(); });
        return img;
    };
    const list = (artifacts) => {
        const list = document.createElement("div");
        list.className = "artifact-list";
        for (const artifact of artifacts) {
            const button = document.createElement("button");
            const name = document.createElement("span");
            button.type = "button";
            button.className = "artifact-chip";
            button.append(icon(artifact));
            name.className = "artifact-chip-name";
            name.textContent = artifact.name || "Attachment";
            button.append(name);
            button.onclick = () => { void openArtifactView(artifact.id, button); };
            list.append(button);
        }
        return list;
    };
    window.nimArtifact = { blob, contentUrl, icon, image, isImage: (artifact) => kind(artifact) === "image", kind, list, mime };
})();

customElements.define("nim-artifact", class extends HTMLElement {
    connectedCallback() {
        const template = document.getElementById("artifact-template");
        this.artifact = JSON.parse(template.dataset.artifact);
        this.annotations = JSON.parse(template.dataset.annotations);
        this.append(template.content.cloneNode(true));
        this.dialog = this.querySelector("dialog");
        this.querySelector("[data-close]").onclick = () => this.dialog.close();
        this.querySelectorAll("[data-tab]").forEach((button) => { button.onclick = () => this.render(button.dataset.tab); });
    }

    label(value) { return String(value).replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase()); }
    percent(value) { return `${Math.round((value || 0) * 100)}%`; }
    confidenceClass(value) {
        const percent = Math.round((value || 0) * 100);
        return percent < 50 ? "is-low" : percent < 80 ? "is-medium" : "is-high";
    }

    formatSize(value) {
        if (!Number.isFinite(value) || value < 0) return "";
        const units = ["B", "KB", "MB", "GB", "TB", "PB"];
        let size = value;
        let unit = 0;
        while (size >= 1024 && unit < units.length - 1) {
            size /= 1024;
            unit++;
        }
        return `${Number(size.toFixed(2))} ${units[unit]}`;
    }
    formatDate(value) { return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(value)); }

    render(tab) {
        this.generation = (this.generation ?? 0) + 1;
        if (tab !== "content") this.clearContent();
        const body = this.querySelector(".message-view-body");
        body.classList.toggle("is-content", tab === "content");
        body.classList.toggle("is-json", tab === "json");
        this.querySelectorAll("[data-tab]").forEach((button) => {
            button.ariaSelected = String(button.dataset.tab === tab);
            button.tabIndex = button.dataset.tab === tab ? 0 : -1;
        });
        body.replaceChildren(tab === "content" ? this.contentView() : tab === "details" ? this.detailsView() : this.jsonView());
    }

    contentView() {
        this.clearContent();
        const generation = this.generation;
        const section = document.createElement("section");
        section.className = "artifact-content";
        const state = document.createElement("p");
        state.className = "artifact-content-state";
        state.textContent = "Loading artifact…";
        section.append(state);
        void window.nimArtifact.blob(this.artifact).then(async (data) => {
            if (!section.isConnected || generation !== this.generation) return;
            const type = window.nimArtifact.mime(this.artifact);
            const url = URL.createObjectURL(data);
            this.previewUrl = url;
            let preview;
            if (type.startsWith("image/")) {
                preview = document.createElement("img");
                preview.alt = this.artifact.name || "Artifact image";
                preview.src = url;
            } else if (type.startsWith("audio/") || type.startsWith("video/")) {
                preview = document.createElement(type.startsWith("audio/") ? "audio" : "video");
                preview.controls = true;
                preview.preload = "metadata";
                preview.src = url;
            } else if (type.startsWith("text/") || ["application/json", "application/xml"].includes(type)) {
                preview = document.createElement("pre");
                preview.textContent = await data.slice(0, 1024 * 1024).text();
                if (data.size > 1024 * 1024) preview.textContent += "\n\nPreview truncated. Download to read the complete file.";
            } else {
                preview = document.createElement("p");
                preview.className = "artifact-content-state";
                preview.textContent = "No preview is available for this file type.";
            }
            if (!section.isConnected || generation !== this.generation) return this.clearContent();
            preview.addEventListener("error", () => {
                URL.revokeObjectURL(url);
                preview.replaceWith(Object.assign(document.createElement("p"), { className: "artifact-content-state", textContent: "This file cannot be previewed." }));
            }, { once: true });
            section.replaceChildren(preview);
        }).catch((error) => {
            if (section.isConnected) state.textContent = error instanceof TypeError ? "Cannot reach the artifact content endpoint." : error.message;
        });
        return section;
    }

    clearContent() {
        if (this.previewUrl) URL.revokeObjectURL(this.previewUrl);
        this.previewUrl = undefined;
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

    detailsView() {
        const section = document.createElement("section");
        section.className = "artifact-details";
        const summary = document.createElement("section");
        summary.className = "message-summary";
        const heading = document.createElement("div");
        const title = document.createElement("h4");
        const confidence = document.createElement("strong");
        const text = document.createElement("p");
        title.textContent = "Summary";
        confidence.textContent = `${this.percent(this.artifact.confidence)} confidence`;
        text.textContent = this.artifact.summary || "No summary available.";
        heading.append(title, confidence);
        summary.append(heading, text);
        const metadata = document.createElement("dl");
        metadata.className = "artifact-metadata";
        for (const [label, value] of [["Type", window.nimArtifact.mime(this.artifact)], ["Size", this.formatSize(this.artifact.size)], ["Created", this.formatDate(this.artifact.created_at)], ["Updated", this.formatDate(this.artifact.updated_at)]].filter(([, value]) => value)) {
            const row = document.createElement("div");
            const key = document.createElement("dt");
            const detail = document.createElement("dd");
            key.textContent = label;
            detail.textContent = value;
            row.append(key, detail);
            metadata.append(row);
        }
        const annotations = document.createElement("section");
        annotations.className = "message-section";
        annotations.append(this.sectionHeader("Annotations", this.annotations.length, this.annotations.length === 1 ? "classification" : "classifications"));
        const table = document.createElement("div");
        table.className = "message-annotation-table";
        if (!this.annotations.length) table.append(Object.assign(document.createElement("p"), { className: "message-empty", textContent: "No annotations linked to this artifact." }));
        for (const annotation of this.annotations) {
            const row = document.createElement("div");
            const line = document.createElement("div");
            const label = document.createElement("span");
            const text = document.createElement("span");
            const value = document.createElement("span");
            row.className = "message-annotation-row";
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
        annotations.append(table);
        section.append(summary, metadata, annotations);
        return section;
    }

    jsonView() {
        const section = document.createElement("section");
        section.className = "message-json";
        const heading = document.createElement("div");
        const title = document.createElement("h4");
        const controls = document.createElement("div");
        title.textContent = "Artifact payload";
        for (const [label, open] of [["Expand all", true], ["Collapse all", false]]) {
            const button = document.createElement("button");
            button.type = "button";
            button.textContent = label;
            button.onclick = () => section.querySelectorAll("details").forEach((detail) => { detail.open = open; });
            controls.append(button);
        }
        const copy = document.createElement("button");
        copy.type = "button";
        copy.textContent = "Copy";
        copy.onclick = () => navigator.clipboard.writeText(JSON.stringify(this.artifact, null, 2));
        controls.append(copy);
        heading.append(title, controls);
        const tree = document.createElement("div");
        tree.className = "message-json-tree";
        tree.tabIndex = 0;
        tree.append(this.treeRoot(this.artifact));
        section.append(heading, tree);
        return section;
    }

    treeRoot(value) {
        const detail = document.createElement("details");
        const summary = document.createElement("summary");
        const children = document.createElement("div");
        detail.open = true;
        summary.textContent = `{${Object.keys(value).length} properties}`;
        children.className = "message-json-children";
        for (const [key, child] of Object.entries(value)) children.append(this.treeNode(key, child));
        detail.append(summary, children);
        return detail;
    }

    treeNode(key, value) {
        if (value !== null && typeof value === "object") {
            const detail = document.createElement("details");
            const summary = document.createElement("summary");
            const name = document.createElement("span");
            const kind = document.createElement("span");
            const children = document.createElement("div");
            name.textContent = `${key}:`;
            kind.textContent = Array.isArray(value) ? `Array(${value.length})` : `{${Object.keys(value).length} properties}`;
            children.className = "message-json-children";
            for (const [childKey, child] of Object.entries(value)) children.append(this.treeNode(childKey, child));
            summary.append(name, kind);
            detail.append(summary, children);
            return detail;
        }
        const line = document.createElement("div");
        const name = document.createElement("span");
        const content = document.createElement("span");
        line.className = "message-json-value";
        name.textContent = `${key}:`;
        content.textContent = value === null ? "null" : typeof value === "string" ? `"${value}"` : String(value);
        content.dataset.type = value === null ? "null" : typeof value;
        line.append(name, content);
        return line;
    }

    open(trigger) {
        this.trigger = trigger;
        this.querySelector("[data-type]").textContent = this.label(window.nimArtifact.kind(this.artifact));
        this.querySelector(".artifact-view-icon").append(window.nimArtifact.icon(this.artifact));
        this.querySelector("[data-id]").textContent = `${this.artifact.id.slice(0, 16)}…`;
        const timestamp = this.querySelector("time");
        timestamp.dateTime = this.artifact.created_at;
        timestamp.textContent = this.formatDate(this.artifact.created_at);
        this.render("content");
        this.dialog.showModal();
        this.prepareDownload();
        this.dialog.addEventListener("close", () => {
            this.generation = (this.generation ?? 0) + 1;
            this.clearContent();
            this.clearDownload();
            this.trigger?.focus();
        }, { once: true });
    }

    prepareDownload() {
        const download = this.querySelector("[data-download]");
        download.removeAttribute("href");
        download.setAttribute("aria-label", "Preparing download");
        download.title = "Preparing download";
        download.setAttribute("aria-disabled", "true");
        void window.nimArtifact.blob(this.artifact).then((data) => {
            if (!this.dialog.open) return;
            this.downloadUrl = URL.createObjectURL(data);
            download.href = this.downloadUrl;
            download.download = this.artifact.name || this.artifact.id;
            download.setAttribute("aria-label", "Download artifact");
            download.title = "Download artifact";
            download.setAttribute("aria-disabled", "false");
        }).catch(() => {
            if (!this.dialog.open) return;
            download.setAttribute("aria-label", "Download unavailable");
            download.title = "Download unavailable";
        });
    }

    clearDownload() {
        if (this.downloadUrl) URL.revokeObjectURL(this.downloadUrl);
        this.downloadUrl = undefined;
    }
});
