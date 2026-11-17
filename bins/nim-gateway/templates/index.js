const socketUrl = new URL("/web/connect", window.location.href);
socketUrl.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
const socket = new WebSocket(socketUrl);

document.addEventListener("alpine:init", () => {
    Alpine.store("nim", { events: [] });
    Alpine.data("nimApp", () => ({
        hud: "idle",
        memoryDisabled: true,
        showChatForm: false,
        devToolsOpen: false,
        draft: "",
        activeAction: null,

        get chat() { return this.$root.querySelector("nim-chat").controller; },
        get memory() { return this.$root.querySelector("nim-memory").controller; },

        setHud(view) {
            if (this.hud === "memory" && view !== "memory") this.memory.close();
            this.hud = view;
            this.showChatForm = view === "panel" && this.activeAction === "chat";
            this.chat.renderChatTranscript();
            if (view === "hub") {
                fetch("/memories?limit=5")
                    .then((response) => {
                        if (!response.ok) throw new Error("Memory request failed");
                        return response.json();
                    })
                    .then((page) => { this.memoryDisabled = !page.data?.length; })
                    .catch(() => { this.memoryDisabled = true; });
            }
        },
        toggleOrb() {
            if (this.hud === "idle") {
                this.setHud("hub");
                this.$nextTick(() => this.$root.querySelector("#chat-action").focus());
            } else if (this.hud === "panel") this.backToHub();
        },
        chooseAction(action) {
            this.activeAction = action.id === "memory-action" ? "memory" : action.id === "chat-action" ? "chat" : "settings";
            if (this.activeAction === "memory") {
                this.setHud("memory");
                this.memory.open();
            } else {
                this.setHud("panel");
                this.chat.openAction(this.activeAction === "chat", action.textContent.trim());
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
        handleKey(event) {
            if (event.isComposing) return;
            if (event.key === "F12") {
                event.preventDefault();
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
