customElements.define("nim-memory-details", class extends HTMLElement {
    renderMemoryArtifacts(memory, host, detailArtifacts, selectionArt, detailArt) {
        const artifacts = memory.record.artifacts ?? [];
        if (!artifacts.length) {
            detailArtifacts.replaceChildren();
            detailArtifacts.hidden = true;
            delete detailArtifacts.dataset.key;
            selectionArt.replaceChildren();
            selectionArt.hidden = true;
            detailArt.replaceChildren();
            detailArt.hidden = true;
            return;
        }
        host.closest("#nim-app").querySelector("nim-artifacts").controller.remember(artifacts);
        const image = artifacts.find(host.closest("#nim-app").querySelector("nim-artifacts").controller.isImage);
        const key = `${memory.id}:${artifacts.map((item) => `${item.id}:${item.version}`).join(",")}`;
        if (detailArtifacts.dataset.key === key) return;
        detailArtifacts.dataset.key = key;
        detailArtifacts.hidden = artifacts.length === 0;
        detailArtifacts.replaceChildren(host.closest("#nim-app").querySelector("nim-artifacts").controller.list(artifacts));
        detailArtifacts.scrollTop = 0;
        selectionArt.replaceChildren();
        detailArt.replaceChildren();
        selectionArt.hidden = !image;
        detailArt.hidden = !image;
        if (image) {
            selectionArt.append(host.closest("#nim-app").querySelector("nim-artifacts").controller.image(image));
            const button = document.createElement("button");
            button.type = "button";
            button.className = "memory-image-open";
            button.setAttribute("aria-label", `View ${image.name || "memory image"}`);
            button.append(host.closest("#nim-app").querySelector("nim-artifacts").controller.image(image));
            button.onclick = () => host.closest("#nim-app").querySelector("nim-artifacts").controller.open(artifacts, artifacts.indexOf(image));
            detailArt.append(button);
        }
    }
});