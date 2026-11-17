const queuedDevEvents = [];
const maxDevEvents = 80;
let nextDevEvent = 0;

function renderEventDetails(container, value) {
    function appendFields(parent, entries) {
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
                appendFields(children, entries);
                row.append(children);
            } else {
                display.textContent = child === null ? "null" : String(child).replaceAll("\n", "\\n").replaceAll("\r", "\\r");
                line.append(label, display);
            }
            parent.append(row);
        }
    }

    const entries = value !== null && typeof value === "object"
        ? Object.entries(value)
        : [["content", value]];
    if (entries.length) {
        appendFields(container, entries);
    } else {
        container.textContent = Array.isArray(value) ? "Empty array" : "Empty object";
    }
}

function eventLabel(subject) {
    const parts = subject.split(".").filter(Boolean);
    if (parts[0] === "nim") parts.shift();
    return parts.map((part) => part.replaceAll("_", " ")).join(" / ") || "Event";
}

function addDevEvent(signal) {
    const date = new Date();
    const subject = typeof signal?.subject === "string" ? signal.subject : "Event";

    const item = {
        key: nextDevEvent++,
        date: date.toISOString(),
        time: date.toLocaleTimeString(),
        subject,
        label: eventLabel(subject),
        content: signal?.content ?? null,
        details: [],
    };
    const events = window.Alpine?.store?.("nim")?.events;
    if (events) {
        events.push(item);
        if (events.length > maxDevEvents) events.shift();
    } else {
        queuedDevEvents.push(item);
        if (queuedDevEvents.length > maxDevEvents) queuedDevEvents.shift();
    }
}

document.addEventListener("alpine:init", () => {
    Alpine.store("nim").events.push(...queuedDevEvents.splice(0));
});

document.querySelector("#dev-tools-events").addEventListener("toggle", (event) => {
    const details = event.target;
    if (!details.open || !details.matches(".dev-event > details")) return;
    const container = details.querySelector(".dev-event-details");
    if (!container) return;
    let tree = container.querySelector(".dev-event-tree");
    if (!tree) {
        tree = document.createElement("div");
        tree.className = "dev-event-tree";
        container.replaceChildren(tree);
    }
    if (tree.dataset.rendered) return;
    const item = Alpine.$data(details).item;
    if (!item) return;
    renderEventDetails(tree, item.content);
    tree.dataset.rendered = "true";
}, true);

socket.addEventListener("message", (event) => {
    try {
        const payload = typeof event.data === "string" ? event.data : new TextDecoder().decode(event.data);
        addDevEvent(JSON.parse(payload));
    } catch {
        // Ignore non-JSON websocket frames in the developer event view.
    }
});
