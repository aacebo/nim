const socketUrl = new URL("/web/connect", window.location.href);
socketUrl.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
class NimSocket extends EventTarget {
    constructor(url) {
        super();
        this.url = url;
        this.binaryType = "arraybuffer";
        this.retryDelay = 1000;
        this.connect();
    }

    connect() {
        const connection = new WebSocket(this.url);
        connection.binaryType = this.binaryType;
        connection.addEventListener("open", () => {
            this.retryDelay = 1000;
            this.dispatchEvent(new Event("open"));
        });
        connection.addEventListener("message", (event) => {
            this.dispatchEvent(new MessageEvent("message", { data: event.data }));
        });
        connection.addEventListener("close", () => {
            this.dispatchEvent(new Event("close"));
            setTimeout(() => this.connect(), this.retryDelay);
            this.retryDelay = Math.min(this.retryDelay * 2, 30000);
        });
    }
}

const socket = new NimSocket(socketUrl);
const pendingMemorySignals = [];
let memorySignalsOverflowed = false;
const devEvents = [];
let nextDevEvent = 0;
let scrollbarHover;
const scrollbarRest = [213, 220, 229, 0.2];
const scrollbarHighlight = [221, 228, 237, 0.36];
const scrollbarColors = new WeakMap();
const scrollbarFrames = new WeakMap();

function scrollbarColor([red, green, blue, alpha]) {
    return `rgb(${red} ${green} ${blue} / ${alpha})`;
}

function animateScrollbarColor(element, target) {
    cancelAnimationFrame(scrollbarFrames.get(element));
    const source = scrollbarColors.get(element) ?? scrollbarRest;
    const start = performance.now();

    function frame(now) {
        const progress = Math.min(1, (now - start) / 180);
        const eased = progress * progress * (3 - 2 * progress);
        const color = source.map((channel, index) => channel + (target[index] - channel) * eased);
        scrollbarColors.set(element, color);
        element.style.setProperty("--scrollbar-thumb", scrollbarColor(color));

        if (progress < 1) scrollbarFrames.set(element, requestAnimationFrame(frame));
        else if (target === scrollbarRest) element.style.removeProperty("--scrollbar-thumb");
    }

    scrollbarFrames.set(element, requestAnimationFrame(frame));
}

function setScrollbarHover(element) {
    if (scrollbarHover === element) return;
    if (scrollbarHover) animateScrollbarColor(scrollbarHover, scrollbarRest);
    scrollbarHover = element;
    if (scrollbarHover) animateScrollbarColor(scrollbarHover, scrollbarHighlight);
}

function scrollbarAtPointer(event) {
    const seen = new Set();

    for (const target of document.elementsFromPoint(event.clientX, event.clientY)) {
        for (let element = target; element instanceof HTMLElement && !seen.has(element); element = element.parentElement) {
            seen.add(element);
            const style = getComputedStyle(element);
            const rect = element.getBoundingClientRect();
            const vertical = element.scrollHeight > element.clientHeight
                && /auto|scroll|overlay/.test(style.overflowY)
                && event.clientX >= rect.right - 10 && event.clientX <= rect.right;
            const horizontal = element.scrollWidth > element.clientWidth
                && /auto|scroll|overlay/.test(style.overflowX)
                && event.clientY >= rect.bottom - 10 && event.clientY <= rect.bottom;

            if (vertical || horizontal) return element;
        }
    }
}

document.addEventListener("pointermove", (event) => setScrollbarHover(scrollbarAtPointer(event)), { passive: true });
document.addEventListener("pointerout", (event) => {
    if (!event.relatedTarget) setScrollbarHover();
}, { passive: true });
document.addEventListener("click", (event) => {
    const dialog = event.target;
    if (!(dialog instanceof HTMLDialogElement) || !dialog.open) return;
    const bounds = dialog.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) {
        dialog.close();
    }
});
socket.addEventListener("message", (event) => {
    try {
        const signal = JSON.parse(typeof event.data === "string" ? event.data : new TextDecoder().decode(event.data));
        const subject = typeof signal.subject === "string" ? signal.subject : "Event";
        const parts = subject.split(".").filter(Boolean);
        if (parts[0] === "nim") parts.shift();
        const events = window.Alpine?.store?.("nim")?.events ?? devEvents;
        events.push({
            key: nextDevEvent++,
            date: new Date().toISOString(),
            time: new Date().toLocaleTimeString(),
            subject,
            label: parts.map((part) => part.replaceAll("_", " ")).join(" / ") || "Event",
            content: signal.content ?? null,
        });
        if (events.length > 100) events.shift();
        if (!signal.subject?.startsWith("nim.memory.")) return;
        if (!document.querySelector("nim-memory")?.controller) {
            pendingMemorySignals.push(signal);
            if (pendingMemorySignals.length > 1000) {
                pendingMemorySignals.shift();
                memorySignalsOverflowed = true;
            }
        }
        if (signal.subject === "nim.memory.create") window.dispatchEvent(new Event("memory-available"));
    } catch {}
});

const viewAssets = {
    artifact: { css: ["message/message.css?v=39", "artifact/artifact.css?v=14"], js: ["artifact/artifact.js?v=8"] },
    chat: {
        css: ["chat/transcript/transcript.css?v=38", "chat/chat.css?v=48"],
        js: [
            "https://unpkg.com/marked@15.0.7/marked.min.js",
            "https://unpkg.com/dompurify@3.2.6/dist/purify.min.js",
            "chat/transcript/transcript.js?v=24",
            "chat/chat.js?v=62",
        ],
    },
    memory: {
        css: ["memory/timeline/timeline.css", "memory/graph/graph.css?v=3", "memory/details/details.css", "memory/memory.css?v=4"],
        js: ["memory/timeline/timeline.js", "memory/graph/graph.js", "memory/details/details.js?v=3", "memory/memory.js?v=10"],
    },
    "dev-tools": { css: ["dev-tools/dev-tools.css?v=6"], js: ["dev-tools/dev-tools.js?v=8"] },
};
const viewLoads = new Map();
const assetLoads = new Map();

function loadAsset(path, type) {
    if (assetLoads.has(path)) return assetLoads.get(path);
    const load = new Promise((resolve, reject) => {
        const element = document.createElement(type === "css" ? "link" : "script");
        const url = path.startsWith("http") ? path : `/web/views/${path}`;
        if (type === "css") {
            element.rel = "stylesheet";
            element.href = url;
        } else {
            element.src = url;
        }
        element.onload = resolve;
        element.onerror = () => { element.remove(); reject(new Error(`Unable to load ${path}`)); };
        document.head.append(element);
    });
    assetLoads.set(path, load);
    load.catch(() => assetLoads.delete(path));
    return load;
}

async function loadArtifactView() {
    await Promise.all(viewAssets.artifact.css.map((path) => loadAsset(path, "css")));
    for (const path of viewAssets.artifact.js) await loadAsset(path, "js");
}

async function openArtifactView(id, trigger) {
    await loadArtifactView();
    const response = await fetch(`/web/views/artifacts/${encodeURIComponent(id)}`);
    if (!response.ok) throw new Error("Unable to load artifact details");
    const documentView = new DOMParser().parseFromString(await response.text(), "text/html");
    const template = documentView.getElementById("artifact-template");
    if (!template) throw new Error("Artifact details are unavailable");
    document.getElementById("artifact-template")?.remove();
    document.querySelector("nim-artifact")?.remove();
    document.body.append(template);
    const view = document.createElement("nim-artifact");
    document.body.append(view);
    view.open(trigger);
}

function loadView(name, path = name) {
    const key = `${name}:${path}`;
    if (viewLoads.has(key)) return viewLoads.get(key);
    const load = (async () => {
        const response = await fetch(path.startsWith("/") ? path : `/web/views/${path}`);
        if (!response.ok) throw new Error(`Unable to load ${name} view`);
        const documentView = new DOMParser().parseFromString(await response.text(), "text/html");
        const template = documentView.getElementById(`${name}-template`);
        if (!template) throw new Error(`Missing ${name} template`);
        document.getElementById(`${name}-template`)?.remove();
        document.body.append(template);
        const state = template.content.querySelector(".action-view")?.dataset ?? template.dataset;
        const hasArtifacts = name === "memory"
            ? JSON.parse(state?.state ?? "[]").some((record) => record.artifacts?.length)
            : name === "chat" && JSON.parse(state?.chat ?? "null")?.messages?.some((message) => JSON.stringify(message.content).includes('"type":"artifact"'));
        if (hasArtifacts) {
            await loadArtifactView();
        }
        await Promise.all(viewAssets[name].css.map((path) => loadAsset(path, "css")));
        for (const path of viewAssets[name].js) {
            await loadAsset(path, "js");
        }
        const element = document.querySelector(`nim-${name}`);
        if (!element || (name === "dev-tools" ? !element.querySelector("#dev-tools") : !element.controller)) {
            throw new Error(`Unable to initialize ${name} view`);
        }
    })();
    viewLoads.set(key, load);
    load.catch(() => viewLoads.delete(key));
    return load;
}

document.addEventListener("alpine:init", () => {
    Alpine.store("nim", { events: devEvents });
    Alpine.data("nimApp", () => ({
        hud: "idle",
        memoryDisabled: true,
        showChatForm: false,
        devToolsOpen: false,
        draft: "",
        viewError: "",
        activeAction: null,
        suppressChatEntry: false,

        init() {
            this.memoryDisabled = this.$root.dataset.hasMemories !== "true";
            const path = window.location.pathname.replace(/^\/web/, "") || "/";
            if (path === "/memory") {
                void this.openRoute("memory");
            } else if (path === "/chat" || path.startsWith("/chat/")) {
                const chatPath = path === "/chat" ? "chat" : `/web/view/chats/${path.slice("/chat/".length)}`;
                void this.openRoute("chat", chatPath, "Chat", path, false);
            }
            window.addEventListener("popstate", () => window.location.reload());
        },
        get chat() { return this.$root.querySelector("nim-chat")?.controller; },
        get memory() { return this.$root.querySelector("nim-memory")?.controller; },

        setHud(view) {
            if (this.hud === "memory" && view !== "memory") this.memory?.close();
            if (view !== "panel" || this.activeAction !== "chat") {
                this.chat?.clearAwaitingResponse();
                this.chat?.closeDrawer();
            }
            document.documentElement.classList.toggle("chat-route", view === "panel" && this.activeAction === "chat");
            this.hud = view;
            this.showChatForm = view === "panel" && this.activeAction === "chat";
            this.chat?.renderChatTranscript();
        },
        toggleOrb() {
            if (this.hud === "idle") {
                this.setHud("hub");
                this.$nextTick(() => this.$root.querySelector("#chat-action").focus());
            } else if (this.hud === "panel") this.backToHub();
        },
        async chooseAction(action) {
            this.activeAction = action.id === "memory-action" ? "memory" : action.id === "chat-action" ? "chat" : "settings";
            const selected = this.activeAction;
            const title = action.textContent.trim();
            this.viewError = "";
            if (selected === "settings") return;
            if (selected === "chat") {
                try {
                    const response = await fetch("/chats?limit=5");
                    const page = response.ok ? await response.json() : null;
                    const chat = page?.data?.[0];
                    if (typeof chat?.id === "string" && chat.id) {
                        await this.openRoute("chat", `/web/view/chats/${encodeURIComponent(chat.id)}`, title, `chat/${encodeURIComponent(chat.id)}`);
                        return;
                    }
                } catch {}
            }
            await this.openRoute(selected, selected === "chat" ? "chat" : "memory", title);
        },
        async openRoute(view, path = view, title = view === "chat" ? "Chat" : "Memory", urlPath = path, animate = true) {
            this.activeAction = view;
            this.viewError = "";
            if (view === "chat" && !animate) this.suppressChatEntry = true;
            try {
                await loadView(view, path);
            } catch (error) {
                console.error(error);
                this.viewError = "Unable to load the view. Please try again.";
                return;
            }
            if (view === "memory") {
                this.setHud("memory");
                this.memory.open();
            } else {
                this.setHud("panel");
                this.chat.openAction(true, title);
            }
            const route = `/web/${urlPath.replace(/^\/+/, "")}`;
            if (window.location.pathname !== route) window.history.pushState({}, "", route);
            if (view === "chat" && !animate) {
                this.$nextTick(() => requestAnimationFrame(() => requestAnimationFrame(() => { this.suppressChatEntry = false; })));
            }
        },
        closeOrbHub() {
            this.setHud("idle");
            if (window.location.pathname !== "/web") window.history.pushState({}, "", "/web");
            this.$nextTick(() => this.$root.querySelector("#orb-trigger").focus());
        },
        backToHub() {
            this.setHud("hub");
            if (window.location.pathname !== "/web") window.history.pushState({}, "", "/web");
            const selector = this.activeAction === "memory" ? "#memory-action" : this.activeAction === "chat" ? "#chat-action" : "#hub-back";
            this.$nextTick(() => this.$root.querySelector(selector).focus());
        },
        setMemoryTopology(topology) { this.memory.setTopology(topology); },
        dismissMemoryDetails() { this.memory.dismissDetails(); },
        expandMemoryGraph() { this.memory.expandSelected(); },
        sendChatMessage() { return this.chat.submitChatMessage(); },
        selectChatFiles(event) { this.chat.handleChatFiles(event.target.files); },
        clearChatError() { this.chat.setChatError(""); },
        closeDevTools() {
            if (this.$refs.devTools?.open) this.$refs.devTools.close();
            this.devToolsOpen = false;
        },
        async handleKey(event) {
            if (event.isComposing) return;
            if (event.key === "F12") {
                event.preventDefault();
                if (!this.$refs.devTools?.open) {
                    try { await loadView("dev-tools"); }
                    catch (error) { console.error(error); this.viewError = "Unable to load developer tools. Please try again."; return; }
                    this.$refs.devTools.showModal();
                    this.devToolsOpen = true;
                    this.$refs.devTools.focus({ preventScroll: true });
                } else {
                    this.closeDevTools();
                }
            } else if (event.key === "Escape") {
                const dialog = document.querySelector("dialog[open]");
                if (dialog) dialog.close();
                else if (!this.chat?.dismissOverlayDrawer()) {
                    if (this.devToolsOpen) this.closeDevTools();
                    else if (this.hud === "memory") {
                        if (!this.memory.dismissDetails()) this.backToHub();
                    } else if (this.hud === "panel" && this.activeAction !== "chat") this.backToHub();
                    else if (this.hud === "hub") this.closeOrbHub();
                    else return;
                }
                event.preventDefault();
            }
        },
    }));
});
