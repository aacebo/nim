customElements.define("nim-memory-graph", class extends HTMLElement {
    makeSvgElement(tag, attributes) {
        const element = document.createElementNS("http://www.w3.org/2000/svg", tag);

        for (const [name, value] of Object.entries(attributes)) {
            element.setAttribute(name, value);
        }

        return element;
    }
});