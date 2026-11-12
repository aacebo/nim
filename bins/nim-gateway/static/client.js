const socketUrl = new URL("/web/connect", window.location.href);
socketUrl.protocol = window.location.protocol === "https:" ? "wss:" : "ws:";

const socket = new WebSocket(socketUrl);

// Shared by the memory inspector and message attachments. Native dialog supplies
// focus containment and makes the underlying HUD inert while an artifact is open.
const artifactView = (() => {
    const records = new Map();
    const blobs = new Map();
    const extensions = { jpg: 'image/jpeg', jpeg: 'image/jpeg', png: 'image/png', gif: 'image/gif', webp: 'image/webp', avif: 'image/avif', svg: 'image/svg+xml', mp3: 'audio/mpeg', wav: 'audio/wav', m4a: 'audio/mp4', ogg: 'audio/ogg', mp4: 'video/mp4', webm: 'video/webm', txt: 'text/plain', md: 'text/plain', csv: 'text/csv', json: 'application/json', yaml: 'text/plain', yml: 'text/plain', log: 'text/plain' };
    function mime(artifact) {
        if (artifact.type === 'text') return 'text/plain';
        const declared = artifact.media_type?.split(';')[0].trim().toLowerCase();
        return declared && declared !== 'application/octet-stream' ? declared
            : extensions[artifact.name?.split('.').pop().toLowerCase()] || 'application/octet-stream';
    }
    const isImage = (artifact) => mime(artifact).startsWith('image/');
    const contentUrl = (artifact) => `/artifacts/${encodeURIComponent(artifact.id)}/content`;
    function remember(items) {
        for (const item of items) {
            if (!item?.id) continue;
            const previous = records.get(item.id);
            if (!previous || (item.version ?? 0) >= (previous.version ?? 0)) records.set(item.id, item);
        }
    }
    async function blob(artifact) {
        const key = `${artifact.id}:${artifact.version ?? 0}`;
        if (!blobs.has(key)) {
            const request = fetch(contentUrl(artifact)).then(async (response) => {
                if (!response.ok) {
                    throw new Error(`Artifact content request failed (HTTP ${response.status}).`);
                }
                const data = await response.blob();
                if (!data.size) throw new Error('The content endpoint returned an empty file.');
                return data.slice(0, data.size, mime(artifact));
            }).catch((error) => { blobs.delete(key); throw error; });
            blobs.set(key, request);
            // Bound retained file data; object URLs belong to individual views.
            if (blobs.size > 12) blobs.delete(blobs.keys().next().value);
        }
        return blobs.get(key);
    }
    function image(artifact, className = '') {
        const img = document.createElement('img');
        img.className = className;
        img.alt = artifact.name || 'Memory image';
        img.decoding = 'async';
        const fail = (error) => {
            const message = error instanceof TypeError ? 'Cannot reach the artifact content endpoint.' : error.message;
            const placeholder = document.createElement('span');
            placeholder.className = 'artifact-image-error';
            placeholder.textContent = message;
            placeholder.title = `${artifact.name || artifact.id}: ${message}`;
            console.warn('Artifact image failed', { id: artifact.id, url: contentUrl(artifact), error });
            img.replaceWith(placeholder);
        };
        void blob(artifact).then((data) => {
            const url = URL.createObjectURL(data);
            img.onload = () => URL.revokeObjectURL(url);
            img.onerror = () => { URL.revokeObjectURL(url); fail(new Error('The downloaded file could not be decoded as an image.')); };
            img.src = url;
        }).catch(fail);
        return img;
    }
    const dialog = document.createElement('dialog');
    dialog.id = 'artifact-view';
    dialog.setAttribute('aria-labelledby', 'artifact-title');
    dialog.innerHTML = `<header class="artifact-header"><div><p class="artifact-kicker">Artifact</p><h2 id="artifact-title"></h2><p id="artifact-meta"></p></div><button type="button" class="artifact-close" aria-label="Close artifact viewer">×</button></header><div class="artifact-body" aria-live="polite"></div><footer class="artifact-footer"><button type="button" class="artifact-prev" aria-label="Previous artifact">← Previous</button><span class="artifact-count"></span><button type="button" class="artifact-next" aria-label="Next artifact">Next →</button><a class="artifact-download" hidden>Download</a></footer>`;
    document.body.append(dialog);
    const body = dialog.querySelector('.artifact-body');
    const download = dialog.querySelector('.artifact-download');
    const previous = dialog.querySelector('.artifact-prev');
    const next = dialog.querySelector('.artifact-next');
    let items = [], index = 0, generation = 0, objectUrl, opener;
    function clear() {
        generation++;
        for (const media of body.querySelectorAll('audio, video')) { media.pause(); media.removeAttribute('src'); media.load(); }
        body.replaceChildren();
        download.hidden = true;
        download.removeAttribute('href');
        if (objectUrl) URL.revokeObjectURL(objectUrl);
        objectUrl = undefined;
    }
    async function render() {
        clear();
        const token = generation;
        const item = records.get(items[index].id) ?? items[index];
        dialog.querySelector('#artifact-title').textContent = item.name || 'Attachment';
        let sizeLabel = '';
        if (Number.isFinite(item.size) && item.size >= 0) {
            const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
            let size = item.size;
            let unit = 0;
            while (size >= 1024 && unit < units.length - 1) {
                size /= 1024;
                unit++;
            }
            sizeLabel = `${Number(size.toFixed(2))} ${units[unit]}`;
        }
        dialog.querySelector('#artifact-meta').textContent = [mime(item), sizeLabel].filter(Boolean).join(' · ');
        dialog.querySelector('.artifact-count').textContent = `${index + 1} / ${items.length}`;
        previous.disabled = index === 0;
        next.disabled = index === items.length - 1;
        body.textContent = 'Loading artifact…';
        try {
            const data = await blob(item);
            if (token !== generation || !dialog.open) return;
            objectUrl = URL.createObjectURL(data);
            download.href = objectUrl;
            download.download = item.name || item.id;
            download.hidden = false;
            const type = mime(item);
            let preview;
            if (type.startsWith('image/')) {
                preview = document.createElement('img');
                preview.alt = item.name || 'Artifact image';
                preview.src = objectUrl;
            } else if (type.startsWith('audio/') || type.startsWith('video/')) {
                preview = document.createElement(type.startsWith('audio/') ? 'audio' : 'video');
                preview.controls = true;
                preview.preload = 'metadata';
                preview.src = objectUrl;
            } else if (type.startsWith('text/') || ['application/json', 'application/xml'].includes(type)) {
                preview = document.createElement('pre');
                // Keep huge logs readable without locking up the page.
                preview.textContent = await data.slice(0, 1024 * 1024).text();
                if (data.size > 1024 * 1024) preview.textContent += '\n\nPreview truncated. Download to read the complete file.';
            } else {
                preview = document.createElement('p');
                preview.textContent = 'No preview for this file type. Download it to open in another app.';
            }
            if (token !== generation || !dialog.open) return;
            preview.addEventListener('error', () => {
                if (token === generation) body.textContent = 'This file cannot be previewed. You can still download it.';
            }, { once: true });
            body.replaceChildren(preview);
        } catch (error) {
            if (token !== generation || !dialog.open) return;
            body.textContent = `${error instanceof TypeError ? 'Cannot reach the artifact content endpoint.' : error.message} `;
            const retry = document.createElement('button');
            retry.type = 'button';
            retry.textContent = 'Retry';
            retry.onclick = render;
            body.append(retry);
        }
    }
    function step(delta) {
        if (index + delta < 0 || index + delta >= items.length) return;
        index += delta;
        void render();
    }
    previous.onclick = () => step(-1);
    next.onclick = () => step(1);
    dialog.querySelector('.artifact-close').onclick = () => dialog.close();
    dialog.addEventListener('click', (event) => {
        const rect = dialog.getBoundingClientRect();
        if (event.target === dialog && (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom)) dialog.close();
    });
    dialog.addEventListener('close', () => { clear(); if (opener?.isConnected) opener.focus({ preventScroll: true }); });
    document.addEventListener('keydown', (event) => {
        if (!dialog.open || event.isComposing) return;
        if (event.key === 'Escape') { event.preventDefault(); event.stopImmediatePropagation(); dialog.close(); }
        if (['ArrowLeft', 'ArrowRight'].includes(event.key) && !event.target.closest('audio, video, input, textarea')) {
            event.preventDefault(); event.stopImmediatePropagation(); step(event.key === 'ArrowLeft' ? -1 : 1);
        }
    }, true);
    function open(artifacts, selected = 0) {
        if (!artifacts.length) return;
        items = artifacts;
        index = selected;
        opener = document.activeElement;
        if (!dialog.open) dialog.showModal();
        void render();
        dialog.querySelector('.artifact-close').focus();
    }
    function artifactIcon(artifact) {
        const type = mime(artifact);
        const extension = artifact.name?.split('.').pop().toLowerCase();
        const kind = type.startsWith('image/') ? 'Image'
            : type.startsWith('audio/') ? 'Audio'
            : type.startsWith('video/') ? 'Video'
            : type === 'application/pdf' || extension === 'pdf' ? 'PDF'
            : /(?:zip|gzip|tar|7z|rar)/.test(type) || ['zip', 'gz', 'tar', '7z', 'rar'].includes(extension) ? 'Archive'
            : type.startsWith('text/') || ['application/json', 'application/xml'].includes(type) ? 'Text' : 'File';
        const paths = {
            Image: 'M3 3h18v18H3z M3 17l6-6 5 5 3-3 4 4 M16 7h.01',
            Audio: 'M9 18V5l11-2v13 M9 8l11-2 M9 18c0 2-6 3-6 0s6-2 6 0 M20 16c0 2-6 3-6 0s6-2 6 0',
            Video: 'M3 5h18v14H3z M10 9l5 3-5 3z',
            Text: 'M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8 M8 16h8',
            PDF: 'M5 2h9l5 5v15H5z M14 2v6h5 M8 17l4-7 4 7 M9 15h6',
            Archive: 'M4 3h16v18H4z M10 3v3h4v3h-4v3h4v3h-4v3h4',
            File: 'M5 2h9l5 5v15H5z M14 2v6h5',
        };
        const icon = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        icon.setAttribute('viewBox', '0 0 24 24');
        icon.setAttribute('aria-hidden', 'true');
        icon.classList.add('artifact-icon');
        const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        path.setAttribute('d', paths[kind]);
        icon.append(path);
        return { icon, kind };
    }
    function list(artifacts) {
        remember(artifacts);
        const list = document.createElement('div');
        list.className = 'artifact-list';
        list.setAttribute('aria-label', 'Artifacts');
        artifacts.forEach((artifact, selected) => {
            const button = document.createElement('button');
            button.type = 'button';
            button.className = 'artifact-chip';
            const { icon, kind } = artifactIcon(artifact);
            const label = document.createElement('span');
            label.className = 'artifact-chip-name';
            label.textContent = artifact.name || 'Attachment';
            button.append(icon, label);
            button.title = `View ${kind.toLowerCase()}: ${artifact.name || 'attachment'}`;
            button.setAttribute('aria-label', button.title);
            button.onclick = () => open(artifacts, selected);
            list.append(button);
        });
        return list;
    }
    return { remember, image, isImage, list, open, get: (id) => records.get(id) };
})();
