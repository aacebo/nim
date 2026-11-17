customElements.define("nim-dev-tools", class extends HTMLElement {
    renderEventDetails(container, value) {
        const entries = value !== null && typeof value === "object"
            ? Object.entries(value)
            : [["content", value]];
        if (entries.length) {
            this.appendFields(container, entries);
        } else {
            container.textContent = Array.isArray(value) ? "Empty array" : "Empty object";
        }
    }

    appendFields(parent, entries) {
        for (const [key, child] of entries) {
            const isContainer = child !== null && typeof child === "object";
            const row = document.createElement(isContainer ? "details" : "div");
            row.className = isContainer ? "dev-tree-branch" : "dev-tree-leaf";
            const line = isContainer ? document.createElement("summary") : row;
            line.className = "dev-tree-line";
            const label = document.createElement("span");
            label.className = "dev-tree-key";
            label.textContent = `${key}:`;
            const display = document.createElement("span");
            display.className = "dev-tree-value";
            if (isContainer) {
                const entries = Object.entries(child);
                display.textContent = Array.isArray(child) ? `Array (${entries.length})` : `Object (${entries.length})`;
                line.append(label, display);
                row.append(line);
                const children = document.createElement("div");
                children.className = "dev-tree-children";
                this.appendFields(children, entries);
                row.append(children);
            } else {
                display.textContent = child === null ? "null" : String(child).replaceAll("\n", "\\n").replaceAll("\r", "\\r");
                line.append(label, display);
            }
            parent.append(row);
        }
    }

    eventLabel(subject) {
        const parts = subject.split(".").filter(Boolean);
        if (parts[0] === "nim") parts.shift();
        return parts.map((part) => part.replaceAll("_", " ")).join(" / ") || "Event";
    }

    addDevEvent(signal) {
        const date = new Date();
        const subject = typeof signal?.subject === "string" ? signal.subject : "Event";

        const item = {
            key: this.nextEvent++,
            date: date.toISOString(),
            time: date.toLocaleTimeString(),
            subject,
            label: this.eventLabel(subject),
            content: signal?.content ?? null,
            details: [],
        };
        const events = window.Alpine?.store?.("nim")?.events;
        if (events) {
            events.push(item);
            if (events.length > 80) events.shift();
        } else {
            this.queuedEvents.push(item);
            if (this.queuedEvents.length > 80) this.queuedEvents.shift();
        }
    }

    connectedCallback() {
        if (!this.hasChildNodes()) {
            this.append(document.getElementById("dev-tools-template").content.cloneNode(true));
        }
        this.listeners = new AbortController();
        this.queuedEvents ??= [];
        this.nextEvent ??= 0;
        const flush = () => Alpine.store("nim").events.push(...this.queuedEvents.splice(0));
        if (window.Alpine) flush();
        else document.addEventListener("alpine:init", flush, { once: true, signal: this.listeners.signal });
        this.querySelector("#dev-tools-events").addEventListener("toggle", (event) => {
            const details = event.target;
            if (!details.open || !details.matches(".dev-event > details")) return;
            const tree = details.querySelector(".dev-event-tree");
            if (!tree || tree.dataset.rendered) return;
            const item = Alpine.$data(details).item;
            if (!item) return;
            this.renderEventDetails(tree, item.content);
            tree.dataset.rendered = "true";
        }, { capture: true, signal: this.listeners.signal });

        socket.addEventListener("message", (event) => {
            try {
                const payload = typeof event.data === "string" ? event.data : new TextDecoder().decode(event.data);
                this.addDevEvent(JSON.parse(payload));
            } catch {
                // Ignore non-JSON websocket frames in the developer event view.
            }
        }, { signal: this.listeners.signal });
    }

    disconnectedCallback() {
        this.listeners?.abort();
    }
});