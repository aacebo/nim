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
socket.addEventListener("message", (event) => {
    try {
        const signal = JSON.parse(typeof event.data === "string" ? event.data : new TextDecoder().decode(event.data));
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
    artifact: { css: ["message/message.css?v=26", "artifact/artifact.css?v=5"], js: ["artifact/artifact.js?v=5"] },
    chat: {
        css: ["chat/transcript/transcript.css?v=13", "chat/chat.css?v=4"],
        js: [
            "https://unpkg.com/marked@15.0.7/marked.min.js",
            "https://unpkg.com/dompurify@3.2.6/dist/purify.min.js",
            "chat/transcript/transcript.js?v=14",
            "chat/chat.js?v=18",
        ],
    },
    memory: {
        css: ["memory/timeline/timeline.css", "memory/graph/graph.css?v=2", "memory/details/details.css", "memory/memory.css?v=3"],
        js: ["memory/timeline/timeline.js", "memory/graph/graph.js", "memory/details/details.js?v=3", "memory/memory.js?v=10"],
    },
    "dev-tools": { css: ["dev-tools/dev-tools.css?v=4"], js: ["dev-tools/dev-tools.js?v=7"] },
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

function loadView(name) {
    if (viewLoads.has(name)) return viewLoads.get(name);
    const load = (async () => {
        const response = await fetch(`/web/views/${name}`);
        if (!response.ok) throw new Error(`Unable to load ${name} view`);
        const documentView = new DOMParser().parseFromString(await response.text(), "text/html");
        const template = documentView.getElementById(`${name}-template`);
        if (!template) throw new Error(`Missing ${name} template`);
        document.getElementById(`${name}-template`)?.remove();
        document.body.append(template);
        const state = template.content.querySelector(".action-view")?.dataset;
        const hasArtifacts = name === "memory"
            ? JSON.parse(state?.state ?? "[]").some((record) => record.artifacts?.length)
            : name === "chat" && JSON.parse(state?.messages ?? "[]").some((message) => JSON.stringify(message.content).includes('"type":"artifact"'));
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
    viewLoads.set(name, load);
    load.catch(() => viewLoads.delete(name));
    return load;
}

document.addEventListener("alpine:init", () => {
    Alpine.store("nim", { events: [] });
    Alpine.data("nimApp", () => ({
        hud: "idle",
        memoryDisabled: true,
        showChatForm: false,
        devToolsOpen: false,
        draft: "",
        viewError: "",
        activeAction: null,

        init() { this.memoryDisabled = this.$root.dataset.hasMemories !== "true"; },
        get chat() { return this.$root.querySelector("nim-chat")?.controller; },
        get memory() { return this.$root.querySelector("nim-memory")?.controller; },

        setHud(view) {
            if (this.hud === "memory" && view !== "memory") this.memory?.close();
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
            try {
                await loadView(selected === "memory" ? "memory" : "chat");
            } catch (error) {
                console.error(error);
                this.viewError = "Unable to load the view. Please try again.";
                return;
            }
            if (this.activeAction !== selected || this.hud !== "hub") return;
            if (this.activeAction === "memory") {
                this.setHud("memory");
                this.memory.open();
            } else {
                this.setHud("panel");
                this.chat.openAction(this.activeAction === "chat", title);
            }
        },
        closeOrbHub() {
            this.setHud("idle");
            this.$nextTick(() => this.$root.querySelector("#orb-trigger").focus());
        },
        backToHub() {
            this.setHud("hub");
            const selector = this.activeAction === "memory" ? "#memory-action" : this.activeAction === "chat" ? "#chat-action" : "#hub-back";
            this.$nextTick(() => this.$root.querySelector(selector).focus());
        },
        setMemoryTopology(topology) { this.memory.setTopology(topology); },
        dismissMemoryDetails() { this.memory.dismissDetails(); },
        expandMemoryGraph() { this.memory.expandSelected(); },
        sendChatMessage() { return this.chat.submitChatMessage(); },
        selectChatFiles(event) { this.chat.handleChatFiles(event.target.files); },
        clearChatError() { this.chat.setChatError(""); },
        closeDevTools() { this.devToolsOpen = false; },
        async handleKey(event) {
            if (event.isComposing) return;
            if (event.key === "F12") {
                event.preventDefault();
                if (!this.devToolsOpen) {
                    try { await loadView("dev-tools"); }
                    catch (error) { console.error(error); this.viewError = "Unable to load developer tools. Please try again."; return; }
                }
                this.devToolsOpen = !this.devToolsOpen;
                if (this.devToolsOpen) this.$nextTick(() => this.$refs.devTools.focus({ preventScroll: true }));
            } else if (event.key === "Escape") {
                if (this.devToolsOpen) this.closeDevTools();
                else if (this.hud === "memory") {
                    if (!this.memory.dismissDetails()) this.backToHub();
                } else if (this.hud === "panel") this.backToHub();
                else if (this.hud === "hub") this.closeOrbHub();
                else return;
                event.preventDefault();
            }
        },
    }));
});
