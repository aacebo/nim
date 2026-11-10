const socketUrl = new URL("/web/connect", window.location.href);
socketUrl.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";

const socket = new WebSocket(socketUrl);
