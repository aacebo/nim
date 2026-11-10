const memoryView = (() => {
    const memories = [
        { id: 1, date: "2025-01-07", type: "Episodic", title: "First light", summary: "A sunrise observation recorded from the east ridge." },
        { id: 2, date: "2025-01-18", type: "Semantic", title: "Signal key", summary: "A glossary of the short pulses used by the survey team." },
        { id: 3, date: "2025-02-03", type: "Procedural", title: "Gate calibration", summary: "The sequence for tuning a relay gate before a crossing." },
        { id: 4, date: "2025-02-19", type: "Episodic", title: "South passage", summary: "A walk through the southern passage during a low-tide survey." },
        { id: 5, date: "2025-03-02", type: "Working", title: "Field notes", summary: "Unsorted observations from three stops along the relay route." },
        { id: 6, date: "2025-03-17", type: "Semantic", title: "Coastal map", summary: "Landmarks and bearings for navigating the coast in low visibility." },
        { id: 7, date: "2025-04-06", type: "Procedural", title: "Relay handoff", summary: "Instructions for passing a signal between survey stations." },
        { id: 8, date: "2025-04-22", type: "Episodic", title: "North station", summary: "A visit to the northern relay and a record of its test transmissions." },
        { id: 9, date: "2025-05-11", type: "Working", title: "Signal drift", summary: "Measurements of a recurring delay in the northern relay." },
        { id: 10, date: "2025-05-29", type: "Semantic", title: "Relay network", summary: "An index of survey stations and how their coverage overlaps." },
        { id: 11, date: "2025-06-10", type: "Episodic", title: "Quiet station", summary: "A standalone observation from a station outside the survey route." },
        { id: 12, date: "2025-06-24", type: "Episodic", title: "Archive crossing", summary: "The final survey crossing, with observations from the coast and relay network." },
    ];
    const fixtureLinks = [[12, 10], [12, 8], [12, 6], [12, 5], [12, 7], [12, 4], [10, 9], [9, 8], [8, 10], [10, 7], [8, 5], [5, 3], [3, 1], [6, 4], [4, 2]];
    const sites = ["East Ridge", "Greywater", "Signal Point", "West Reach", "Beacon Hill", "Salt Flats", "Outer Shoal", "Glass Harbor"];

    for (let id = memories.length + 1; id <= 1000; id++) {
        const index = id - 13;
        const site = sites[Math.floor(index / 12) % sites.length];
        const record = memories[index % 12];
        const date = new Date(Date.UTC(2025, 5, 24 + (index + 1) * 2)).toISOString().slice(0, 10);
        const visit = Math.floor(index / (12 * sites.length)) + 1;
        memories.push({ id, date, type: record.type, title: `${record.title} / ${site}`, summary: `${site}, survey ${visit}. ${record.summary}`, art: index % 12 });
        fixtureLinks.push([id, id - 1]);

        if (id > 24) {
            fixtureLinks.push([id, id - 12], [id, id - (3 + id % 5)]);
        }

        if (id > 60 && id % 3 === 0) {
            fixtureLinks.push([id, id - 36]);
        }
    }

    const relations = new Map();

    for (const [from, to] of fixtureLinks) {
        for (const [id, neighbor] of [[from, to], [to, from]]) {
            if (!relations.has(id)) {
                relations.set(id, new Set());
            }

            relations.get(id).add(neighbor);
        }
    }

    const memoryById = new Map(memories.map((memory) => [memory.id, memory]));
    const dateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeZone: "UTC" });
    const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
    const view = document.querySelector("#memory-view");
    const timelineStage = document.querySelector("#memory-timeline-stage");
    const timeline = document.querySelector("#memory-timeline");
    const strand = document.querySelector("#memory-strand");
    const timelineCanvas = document.querySelector("#memory-timeline-strand");
    const timelineContext = timelineCanvas.getContext("2d");
    const selectionIndex = document.querySelector("#memory-selection-index");
    const selectionTitle = document.querySelector("#memory-selection-title");
    const selectionType = document.querySelector("#memory-selection-type");
    const selectionDate = document.querySelector("#memory-selection-date");
    const graph = document.querySelector("#memory-graph");
    const graphSurface = graph.querySelector(".memory-graph-surface");
    const edgeLayer = document.querySelector("#memory-edges");
    const nodeLayer = document.querySelector("#memory-nodes");
    const timelineToggle = document.querySelector("#memory-timeline-toggle");
    const graphToggle = document.querySelector("#memory-graph-toggle");
    const viewStatus = document.querySelector("#memory-view-status");
    const details = document.querySelector("#memory-details");
    const detailTitle = document.querySelector("#memory-detail-title");
    const detailArt = document.querySelector("#memory-detail-art");
    const detailMeta = document.querySelector("#memory-detail-meta");
    const detailSummary = document.querySelector("#memory-detail-summary");
    const detailClose = document.querySelector("#memory-detail-close");
    const expand = document.querySelector("#memory-expand");
    const timelineFocus = 0.64;
    const timelineSliceCount = 152;
    const timelineButtons = new Map();
    const graphNodes = new Map();
    const graphEdges = new Map();
    const center = { x: 520, y: 308 };
    const graphSlots = [];
    const emptyRelations = new Set();
    const memoryArtPaths = [
        "M4 60 24 38 34 50 48 29 67 55 79 43 96 60Z",
        "M29 24h42v39H29zM37 33h26M37 41h18M37 49h24M37 57h13",
        "M20 65V46h8V25h9v13h26V25h9v21h8v19H63V50H37v15Z",
        "M10 67 25 38h11l6-15h16l8 15h12l12 29H67V50H33v17Z",
        "M22 24 66 20l11 12-3 34-48 3Zm43-4v14l12-2M34 37l22-2M34 46l30-2M34 55l20-2",
        "M11 58 25 43l12 6 13-14 14 7 14-18 12 8-2 29-20 8-18-7-16 7-16-8Z",
        "M23 65h54v-7H64L57 32H43l-7 26H23ZM50 19v14M35 22l7 10m23-10-7 10",
        "M16 62h68v-6H73V43H60V28H40v15H27v13H16ZM46 38h8m-8 8h8M32 51h8m20 0h8",
        "M12 53h14l7-12 9 21 9-29 10 27 8-17 7 10h12v10H12Z",
        "M16 65h68v-6H73V44h-9v15H55V31H45v28h-9V44h-9v15H16ZM30 42l20-16 19 16",
        "M15 65h70v-5H69V43H57v-7H43v7H31v17H15ZM46 50h8v10h-8Z",
        "M8 65h84v-7H76V44h-8v14H60V34H40v24h-8V44h-8v14H8ZM47 42h6v16h-6Z",
    ];
    let selectedId = memories.at(-1).id;
    let topology = "timeline";
    let detailAnchor;
    let hoveredNode;
    let focusedNode;
    let graphFrame;
    let artSequence = 0;
    let timelinePitch;
    let timelineFrame;
    let timelineOpenedAt;
    let lastTimelineDraw;
    let timelineHover;
    let timelineLift = 0;
    let timelineLiftAt = -1;
    let timelineWidth;
    let timelineHeight;
    let timelineScale;
    let timelineSlices = [];
    let graphRows = 0;
    let visibleMarkers = new Set();

    function selectMemory(id) {
        const previous = timelineButtons.get(selectedId);

        if (previous) {
            previous.tabIndex = -1;
            previous.setAttribute("aria-current", "false");
        }

        selectedId = id;
        const memory = memoryById.get(id);
        selectionIndex.textContent = String(id).padStart(2, "0");
        selectionTitle.textContent = memory.title;
        selectionType.textContent = memory.type;
        selectionDate.textContent = dateFormat.format(new Date(memory.date));

        const button = timelineButtons.get(id);

        if (button) {
            button.tabIndex = 0;
            button.setAttribute("aria-current", "true");
        }

        for (const [memoryId, node] of graphNodes) {
            node.button.setAttribute("aria-current", String(memoryId === id));
        }

        for (const { line, fromId, toId } of graphEdges.values()) {
            line.classList.toggle("memory-link-active", fromId === id || toId === id);
        }
    }

    function timelinePoint(progress, width, height) {
        const left = progress < 0.52;
        const t = left ? progress / 0.52 : (progress - 0.52) / 0.48;
        const u = 1 - t;
        const x = left
            ? 0.18 * u ** 3 + 0.24 * 3 * u ** 2 * t + 0.38 * 3 * u * t ** 2 + 0.57 * t ** 3
            : 0.57 * u ** 3 + 0.76 * 3 * u ** 2 * t + 0.78 * 3 * u * t ** 2 + 1.08 * t ** 3;
        const y = left
            ? 1.2 * u ** 3 + 0.72 * 3 * u ** 2 * t + 0.49 * 3 * u * t ** 2 + 0.62 * t ** 3
            : 0.62 * u ** 3 + 0.75 * 3 * u ** 2 * t + 0.17 * 3 * u * t ** 2 - 0.12 * t ** 3;
        const depth = 0.52 - 0.83 * progress + 1.48 * Math.max(0, 1 - progress) ** 2.5;
        return { x: (x - 0.5) * width * (1 + depth), y: (y - 0.5) * height * (1 + depth), z: depth * width };
    }

    function projectTimeline({ x, y, z }, width, height) {
        const scale = width / (width + z);
        return { x: width / 2 + x * scale, y: height / 2 + y * scale };
    }

    function timelineSlice(progress, width, height, time, lift) {
        const point = timelinePoint(progress, width, height);
        const next = timelinePoint(progress + 0.001, width, height);
        const dx = next.x - point.x;
        const dy = next.y - point.y;
        const dz = next.z - point.z;
        const length = Math.hypot(dx, dy, dz);
        const planar = Math.hypot(dx, dy);
        const nx = -dy / planar;
        const ny = dx / planar;
        const radius = Math.min(width * 0.10, height * 0.14) * (1 + lift * 0.22);
        const twist = progress * 7.1 + time * 0.42;
        const corners = [];

        for (let side = 0; side < 6; side++) {
            const angle = side * Math.PI / 3 + twist;
            const a = Math.cos(angle) * radius;
            const b = Math.sin(angle) * radius * 0.76;
            corners.push(projectTimeline({
                x: point.x + nx * a - dz * ny / length * b - nx * lift * 23,
                y: point.y + ny * a + dz * nx / length * b - ny * lift * 23,
                z: point.z + planar / length * b - lift * width * 0.035,
            }, width, height));
        }

        return corners;
    }

    function traceTimeline(points) {
        timelineContext.moveTo(points[0].x, points[0].y);

        for (let index = 1; index < points.length; index++) {
            timelineContext.lineTo(points[index].x, points[index].y);
        }
    }

    function drawTimeline(now) {
        timelineFrame = undefined;

        if (view.hidden || timelineStage.hidden) {
            return;
        }

        const still = reducedMotion.matches;
        const elapsed = Math.max(0, (now - timelineOpenedAt) / 1000);

        if (still || now - lastTimelineDraw >= 30 || !lastTimelineDraw) {
            const width = timelineWidth;
            const height = timelineHeight;
            const time = still ? 0 : elapsed;
            const reveal = still ? 1 : Math.min(1, elapsed / 1.08);
            const limit = reveal * reveal * (3 - 2 * reveal);
            const selected = timelineFocus + ((selectedId - 1) * timelinePitch - timeline.scrollLeft) / width;

            if (timelineHover) {
                timelineLiftAt = Number(timelineHover.dataset.progress);
            }

            timelineLift += ((timelineHover ? 1 : 0) - timelineLift) * (still ? 1 : 0.24);
            timelineContext.clearRect(0, 0, width, height);
            timelineContext.lineJoin = "round";

            const slices = [];

            for (let index = 0; index <= timelineSliceCount; index++) {
                const progress = index / timelineSliceCount;

                if (progress > limit) {
                    break;
                }

                const lift = Math.max(0, 1 - Math.abs(progress - timelineLiftAt) / 0.024) * timelineLift;
                slices.push(timelineSlice(progress, width, height, time, lift));
            }

            timelineSlices = slices;

            const glow = timelineContext.createLinearGradient(0, height, width, 0);
            glow.addColorStop(0, "rgb(255 255 255 / 13%)");
            glow.addColorStop(0.38, "rgb(255 255 255 / 42%)");
            glow.addColorStop(0.57, "rgb(255 255 255 / 9%)");
            glow.addColorStop(1, "rgb(255 255 255 / 28%)");
            timelineContext.beginPath();

            for (let index = 0; index < slices.length; index += 2) {
                if (Math.abs(index / timelineSliceCount - selected) > 0.04) {
                    traceTimeline(slices[index]);
                    timelineContext.closePath();
                }
            }

            timelineContext.strokeStyle = glow;
            timelineContext.lineWidth = 2.2;
            timelineContext.shadowColor = "#fff";
            timelineContext.shadowBlur = 11 * timelineScale;
            timelineContext.stroke();
            timelineContext.shadowBlur = 0;

            timelineContext.beginPath();

            for (const side of [0, 2, 4]) {
                traceTimeline(slices.map((slice) => slice[side]));
            }

            timelineContext.strokeStyle = "rgb(255 255 255 / 19%)";
            timelineContext.lineWidth = 1.4;
            timelineContext.stroke();

            for (let index = 0; index < slices.length; index++) {
                const progress = index / timelineSliceCount;
                const distance = Math.abs(progress - selected);
                const active = distance < 0.039;
                const cap = distance < 0.006;
                const highlight = Math.abs(progress - timelineLiftAt) < 0.012 && timelineLift > 0.1;
                const slice = slices[index];
                timelineContext.beginPath();
                traceTimeline(slice);
                timelineContext.closePath();
                timelineContext.fillStyle = active ? "rgb(42 6 16 / 19%)" : "rgb(251 248 250 / 4%)";

                if (cap) {
                    const left = Math.min(...slice.map(({ x }) => x));
                    const right = Math.max(...slice.map(({ x }) => x));
                    const crimson = timelineContext.createLinearGradient(left, 0, right, 0);
                    crimson.addColorStop(0, "rgb(192 43 56 / 93%)");
                    crimson.addColorStop(0.47, "rgb(94 8 24 / 92%)");
                    crimson.addColorStop(1, "rgb(63 13 26 / 88%)");
                    timelineContext.fillStyle = crimson;
                }

                timelineContext.fill();
                timelineContext.strokeStyle = cap ? "#ffe6e7" : active ? "rgb(231 109 120 / 68%)" : "rgb(255 254 255 / 52%)";

                if (highlight && !active) {
                    timelineContext.strokeStyle = `rgb(255 254 255 / ${0.52 + timelineLift * 0.45})`;
                }

                timelineContext.lineWidth = cap ? 2 : active ? 0.9 : 0.72;
                timelineContext.stroke();
            }

            if (selected >= 0 && selected <= limit) {
                const anchor = projectTimeline(timelinePoint(selected, width, height), width, height);
                const top = anchor.y - Math.min(height * 0.23, width * 0.18);
                timelineContext.beginPath();
                traceTimeline([{ x: anchor.x, y: top }, { x: anchor.x, y: anchor.y - Math.min(height * 0.08, width * 0.07) }]);
                timelineContext.strokeStyle = "#8e1224";
                timelineContext.lineWidth = 1.15;
                timelineContext.stroke();
                timelineContext.fillStyle = "#a01e2e";
                timelineContext.beginPath();
                traceTimeline([{ x: anchor.x, y: top - 8 }, { x: anchor.x + 5, y: top - 3 }, { x: anchor.x, y: top + 2 }, { x: anchor.x - 5, y: top - 3 }]);
                timelineContext.closePath();
                timelineContext.fill();
            }

            lastTimelineDraw = now;
        }

        if (!still) {
            timelineFrame = requestAnimationFrame(drawTimeline);
        }
    }

    function refreshTimeline() {
        if (!view.hidden && !timelineStage.hidden && timelineFrame === undefined) {
            timelineFrame = requestAnimationFrame(drawTimeline);
        }
    }

    function stopTimeline() {
        cancelAnimationFrame(timelineFrame);
        timelineFrame = undefined;
        timelineHover = undefined;
        timeline.removeAttribute("data-hovering");
        timelineSlices = [];
        timelineLift = 0;
    }

    function makeSvgElement(tag, attributes) {
        const element = document.createElementNS("http://www.w3.org/2000/svg", tag);

        for (const [name, value] of Object.entries(attributes)) {
            element.setAttribute(name, value);
        }

        return element;
    }

    function drawHorizonPoints() {
        const layer = document.querySelector("#memory-horizon-points");
        const floor = [];
        const lights = [];
        let seed = 17;
        const random = () => {
            seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
            return seed / 4294967296;
        };

        for (let row = 0; row < 39; row++) {
            const depth = (row + 1) / 39;
            const y = 443 + 375 * depth ** 2.6;
            const spacing = 7 + 44 * depth ** 1.5;

            for (let column = -Math.ceil(600 / spacing); column <= Math.ceil(600 / spacing); column++) {
                if (random() < 0.13) {
                    continue;
                }

                floor.push(makeSvgElement("circle", {
                    cx: 600 + column * spacing + (row % 2) * spacing / 2,
                    cy: y,
                    r: (0.36 + depth * 0.76).toFixed(2),
                    opacity: (0.08 + random() * 0.55 * (1 - depth * 0.53)).toFixed(2),
                }));
            }
        }

        for (let index = 0; index < 860; index++) {
            const x = random() * 1200;
            const horizon = 440 + 6 * Math.sin(x / 150) + 3 * Math.cos(x / 55);
            const spread = random() ** 2.25;
            const y = horizon + (random() < 0.53 ? 1 : -1) * (spread * 68 + random() * 3);
            const radius = 0.34 + random() ** 3 * 1.3;
            const opacity = (0.24 + random() * 0.68) * (1 - spread * 0.57);
            const fill = index % 5 === 0 ? "#b2d1f2" : index % 3 === 0 ? "#ffc077" : "#fff0c6";

            if (radius > 0.84) {
                lights.push(makeSvgElement("circle", {
                    cx: x, cy: y, r: radius * 4.6, opacity: opacity * 0.68, fill: "url(#memory-horizon-bloom)",
                }));
            }

            lights.push(makeSvgElement("circle", { cx: x, cy: y, r: radius, opacity, fill }));
        }

        const floorLayer = makeSvgElement("g", { class: "memory-floor-points" });
        floorLayer.append(...floor);
        layer.replaceChildren(floorLayer, ...lights);
    }

    function measureTimeline() {
        const width = timeline.clientWidth;
        timelineWidth = width;
        timelineHeight = timeline.clientHeight;
        timelineScale = Math.min(2, window.devicePixelRatio);
        timelineCanvas.width = Math.round(timelineWidth * timelineScale);
        timelineCanvas.height = Math.round(timelineHeight * timelineScale);
        timelineContext.setTransform(timelineScale, 0, 0, timelineScale, 0, 0);
        const nodeWidth = strand.firstElementChild.getBoundingClientRect().width;
        const focus = width * timelineFocus;
        timelinePitch = nodeWidth + parseFloat(getComputedStyle(strand).columnGap);
        strand.style.paddingInlineStart = `${focus - nodeWidth / 2}px`;
        strand.style.paddingInlineEnd = `${width - focus - nodeWidth / 2}px`;
    }

    function positionTimelineTiles(followSelection = false) {
        const { clientWidth: width, clientHeight: height, scrollLeft } = timeline;
        const focalX = width * timelineFocus;
        const nearestId = Math.min(memories.length, Math.max(1, Math.round(scrollLeft / timelinePitch) + 1));
        const first = Math.max(0, Math.ceil((scrollLeft - focalX) / timelinePitch));
        const last = Math.min(memories.length - 1, Math.floor((scrollLeft - focalX + width) / timelinePitch));
        const markers = new Set();
        const focused = document.activeElement;

        for (let index = first; index <= last; index++) {
            const button = timelineButtons.get(index + 1);
            const x = focalX + index * timelinePitch - scrollLeft;
            const progress = x / width;
            const position = projectTimeline(timelinePoint(progress, width, height), width, height);
            button.style.setProperty("--memory-y", `${position.y}px`);
            button.style.setProperty("--memory-x", `${position.x - x}px`);
            button.dataset.progress = progress;
            button.hidden = false;
            markers.add(button);
        }

        for (const button of visibleMarkers) {
            if (!markers.has(button)) {
                button.hidden = true;
            }
        }

        visibleMarkers = markers;

        if (timelineHover?.hidden) {
            setTimelineHover(undefined);
        }

        if (followSelection && details.hidden && nearestId !== selectedId) {
            selectMemory(nearestId);

            if (focused.hidden && timeline.contains(focused)) {
                timelineButtons.get(nearestId).focus({ preventScroll: true });
            }
        }

        refreshTimeline();
    }

    function centerTimeline(id) {
        timeline.scrollLeft = (id - 1) * timelinePitch;
        positionTimelineTiles();
    }

    function centerGraph(id) {
        const { x, y } = graphNodes.get(id);
        const surface = graphSurface.getBoundingClientRect();
        const viewport = graph.getBoundingClientRect();
        graph.scrollLeft += surface.left - viewport.left + x - graph.clientWidth / 2;
        graph.scrollTop += surface.top - viewport.top + y - graph.clientHeight / 2;
    }

    function updateStatus() {
        if (topology === "timeline") {
            viewStatus.textContent = `${memories.length} records · ← → Browse`;
        } else if (graphNodes.size === 1) {
            viewStatus.textContent = "No linked memories in this demo.";
        } else {
            viewStatus.textContent = `${graphNodes.size} of ${memories.length} memories · ${graphEdges.size} links`;
        }
    }

    function relatedIds(id) {
        return relations.get(id) ?? emptyRelations;
    }

    function makeMemoryArt(memory) {
        const art = makeSvgElement("svg", {
            class: "memory-art",
            viewBox: "0 0 100 80",
            preserveAspectRatio: "xMidYMid slice",
            "aria-hidden": "true",
        });
        const id = `memory-art-${++artSequence}`;
        const defs = makeSvgElement("defs", {});
        const shade = makeSvgElement("linearGradient", { id: `${id}-shade`, x1: "0", y1: "0", x2: "0.78", y2: "1" });
        shade.append(
            makeSvgElement("stop", { "stop-color": "#d7dce0" }),
            makeSvgElement("stop", { offset: ".38", "stop-color": "#879098" }),
            makeSvgElement("stop", { offset: ".72", "stop-color": "#3e4852" }),
            makeSvgElement("stop", { offset: "1", "stop-color": "#222c36" }),
        );
        const shape = memoryArtPaths[memory.art ?? memory.id - 1];
        const mask = makeSvgElement("clipPath", { id: `${id}-shape` });
        mask.append(makeSvgElement("path", { d: shape }));
        defs.append(shade, mask);

        const horizon = 45 + (memory.id % 4) * 3;
        const distance = makeSvgElement("path", {
            class: "memory-art-distance",
            d: `M0 ${horizon}l13-5 8 2 10-8 14 5 14-11 11 6 9-4 21 9v38H0Z`,
        });
        const land = makeSvgElement("path", { class: "memory-art-land", d: "M0 63 22 55 34 58 57 49 75 56 100 52v28H0Z" });
        const ground = makeSvgElement("path", {
            class: "memory-art-detail",
            d: "M0 66q32-5 62 0t38-2M0 71q38-4 66-1t34-2M0 77q40-6 68-2t32-3M10 80l30-21m8 21 9-22m27 22-15-22",
        });
        const shadow = makeSvgElement("path", { class: "memory-art-shadow", d: shape, transform: "translate(4 3)" });
        const subject = makeSvgElement("path", { class: "memory-art-subject", d: shape, fill: `url(#${id}-shade)` });
        const detail = makeSvgElement("path", {
            class: "memory-art-texture",
            "clip-path": `url(#${id}-shape)`,
            d: Array.from({ length: 17 }, (_, index) => {
                const y = 22 + index * 3;
                const x = 12 + ((memory.id * 19 + index * 11) % 68);
                return `M0 ${y}l100 ${-9 + (memory.id % 4) * 4}M${x} ${y}v11`;
            }).join(""),
        });

        art.append(defs, distance, land, ground, shadow, subject, detail);
        return art;
    }

    function addGraphNode(id, position) {
        const memory = memoryById.get(id);
        const button = document.createElement("button");
        const frame = document.createElement("span");
        const label = document.createElement("span");
        button.className = "memory-graph-node";
        button.type = "button";
        frame.className = "memory-graph-frame";
        label.className = "memory-graph-label";
        label.textContent = memory.title;
        frame.append(makeMemoryArt(memory));
        button.append(frame, label);
        button.style.left = `${position.x}px`;
        button.style.top = `${position.y}px`;
        button.setAttribute("aria-current", String(id === selectedId));
        button.addEventListener("pointerenter", (event) => {
            if (event.pointerType === "mouse") {
                hoveredNode = id;
                moveGraphNodes();
            }
        });
        button.addEventListener("pointerleave", () => {
            if (hoveredNode === id) {
                hoveredNode = undefined;
                moveGraphNodes();
            }
        });
        button.addEventListener("focus", () => {
            if (button.matches(":focus-visible")) {
                focusedNode = id;
                moveGraphNodes();
            }
        });
        button.addEventListener("blur", () => {
            if (focusedNode === id) {
                focusedNode = undefined;
                moveGraphNodes();
            }
        });
        button.addEventListener("click", () => showDetails(id, button));
        nodeLayer.append(button);
        graphNodes.set(id, { ...position, dx: 0, dy: 0, scale: 1, button });
    }

    function addGraphRows(count) {
        for (let row = graphRows; row < graphRows + count; row++) {
            for (let column = 0; column < 9; column++) {
                const x = 83 + column * 106 + (row % 2) * 40 + Math.sin(row * 4 + column * 5) * 14;
                const y = 77 + row * 110 + Math.cos(row * 3 - column * 4) * 17;

                if (Math.hypot(x - center.x, y - center.y) > 75) {
                    graphSlots.push({ x, y });
                }
            }
        }

        graphRows += count;
        const height = Math.max(620, graphRows * 110 + 50);
        graphSurface.style.height = `${height}px`;
        edgeLayer.setAttribute("viewBox", `0 0 1040 ${height}`);
    }

    function addGraphNeighbors(id) {
        const anchor = graphNodes.get(id);

        for (const neighbor of relatedIds(id)) {
            if (graphNodes.has(neighbor)) {
                continue;
            }

            if (!graphSlots.length) {
                addGraphRows(3);
            }

            graphSlots.sort((a, b) => Math.hypot(a.x - anchor.x, a.y - anchor.y) - Math.hypot(b.x - anchor.x, b.y - anchor.y));
            addGraphNode(neighbor, graphSlots.shift());
        }
    }

    function expandGraph(id, newRoot = false) {
        addGraphNeighbors(id);

        if (newRoot) {
            for (let depth = 1; depth < 3; depth++) {
                for (const neighbor of [...graphNodes.keys()]) {
                    addGraphNeighbors(neighbor);
                }
            }
        }

        for (const [fromId, from] of graphNodes) {
            for (const toId of relatedIds(fromId)) {
                const to = graphNodes.get(toId);
                const key = `${fromId}:${toId}`;

                if (fromId > toId || !to || graphEdges.has(key)) {
                    continue;
                }

                const line = makeSvgElement("line", {});
                const joint = makeSvgElement("rect", { class: "memory-link-point", x: -1.5, y: -1.5, width: 3, height: 3 });
                line.classList.toggle("memory-link-active", fromId === selectedId || toId === selectedId);
                edgeLayer.append(line, joint);
                graphEdges.set(key, { line, joint, fromId, toId, from, to });
                positionGraphEdge(line, joint, from, to);
            }
        }

        updateStatus();
        moveGraphNodes();
    }

    function resetGraph(id) {
        clearGraphFocus();
        graphNodes.clear();
        graphEdges.clear();
        nodeLayer.replaceChildren();
        edgeLayer.replaceChildren();
        graphSlots.length = 0;
        graphRows = 0;
        addGraphRows(5);
        addGraphNode(id, center);
    }

    function clearGraphFocus() {
        hoveredNode = undefined;
        focusedNode = undefined;
        cancelAnimationFrame(graphFrame);
    }

    function positionGraphEdge(line, joint, from, to) {
        const x1 = from.x + from.dx;
        const y1 = from.y + from.dy;
        const x2 = to.x + to.dx;
        const y2 = to.y + to.dy;
        line.setAttribute("x1", x1);
        line.setAttribute("y1", y1);
        line.setAttribute("x2", x2);
        line.setAttribute("y2", y2);
        joint.setAttribute("transform", `translate(${(x1 + x2) / 2} ${(y1 + y2) / 2}) rotate(45)`);
    }

    function moveGraphNodes() {
        cancelAnimationFrame(graphFrame);
        const anchor = graphNodes.get(hoveredNode ?? focusedNode);
        const moves = [...graphNodes.values()].map((node) => {
            let x = 0;
            let y = 0;

            if (anchor && anchor !== node) {
                const dx = node.x - anchor.x;
                const dy = node.y - anchor.y;
                const distance = Math.hypot(dx, dy);
                const push = Math.max(0, 38 * (1 - distance / 410));
                x = dx / distance * push;
                y = dy / distance * push;
            }

            return { node, x, y, dx: node.dx, dy: node.dy, scale: node.scale, targetScale: anchor === node ? 1.55 : 1 };
        }).filter((move) => move.x !== move.dx || move.y !== move.dy || move.scale !== move.targetScale);
        const movingNodes = new Set(moves.map(({ node }) => node));
        const movingEdges = [...graphEdges.values()].filter(({ from, to }) => movingNodes.has(from) || movingNodes.has(to));
        const start = performance.now();

        function draw(now) {
            const progress = reducedMotion.matches || graph.hidden ? 1 : Math.min(1, (now - start) / 180);
            const ease = 1 - (1 - progress) ** 3;

            for (const move of moves) {
                const { node } = move;
                node.dx = move.dx + (move.x - move.dx) * ease;
                node.dy = move.dy + (move.y - move.dy) * ease;
                node.scale = move.scale + (move.targetScale - move.scale) * ease;
                node.button.style.transform = `translate(${node.dx}px, ${node.dy}px) scale(${node.scale})`;
                node.button.style.zIndex = anchor === node ? "1" : "";
            }

            for (const { line, joint, from, to } of movingEdges) {
                positionGraphEdge(line, joint, from, to);
            }

            positionDetails();

            if (progress < 1 && moves.length) {
                graphFrame = requestAnimationFrame(draw);
            }
        }

        draw(start);
    }

    function positionDetails() {
        if (details.hidden) {
            return;
        }

        const inset = 12;
        const anchor = (detailAnchor.querySelector(".memory-graph-frame") ?? detailAnchor).getBoundingClientRect();
        const { width, height } = details.getBoundingClientRect();
        const right = view.clientWidth - inset;
        const bottom = view.clientHeight - inset;
        let x = anchor.right + inset;
        let y = anchor.top;

        if (x + width > right) {
            x = anchor.left - width - inset;

            if (x < inset) {
                x = anchor.left + (anchor.width - width) / 2;
                y = anchor.bottom + inset + height <= bottom ? anchor.bottom + inset : anchor.top - height - inset;
            }
        }

        details.style.left = `${Math.max(inset, Math.min(x, right - width))}px`;
        details.style.top = `${Math.max(inset, Math.min(y, bottom - height))}px`;
    }

    function showDetails(id, button) {
        const memory = memoryById.get(id);
        selectMemory(id);
        detailAnchor = button;
        details.dataset.topology = topology;
        detailTitle.textContent = memory.title;
        detailMeta.textContent = `${dateFormat.format(new Date(memory.date))} · ${memory.type}`;
        detailSummary.textContent = memory.summary;
        detailArt.hidden = topology !== "graph";
        detailArt.replaceChildren(...(detailArt.hidden ? [] : [makeMemoryArt(memory)]));
        expand.hidden = detailArt.hidden || relatedIds(id).size === 0;
        details.hidden = false;
        positionDetails();
        detailClose.focus();
    }

    function dismissDetails(restoreFocus = true) {
        if (details.hidden) {
            return false;
        }

        details.hidden = true;

        if (restoreFocus) {
            detailAnchor.focus({ preventScroll: true });
        }

        return true;
    }

    function setTopology(next) {
        dismissDetails(false);
        clearGraphFocus();
        topology = next;
        timelineStage.hidden = next !== "timeline";
        graph.hidden = next !== "graph";
        timelineToggle.setAttribute("aria-pressed", String(!timelineStage.hidden));
        graphToggle.setAttribute("aria-pressed", String(!graph.hidden));

        stopTimeline();

        if (graph.hidden) {
            timelineOpenedAt = performance.now();
            lastTimelineDraw = undefined;
            measureTimeline();
            centerTimeline(selectedId);
        } else {
            const newRoot = !graphNodes.has(selectedId);

            if (newRoot) {
                resetGraph(selectedId);
            }

            expandGraph(selectedId, newRoot);
            centerGraph(selectedId);
        }

        updateStatus();
    }

    function buildTimeline() {
        const items = document.createDocumentFragment();

        for (const memory of memories) {
            const item = document.createElement("li");
            const button = document.createElement("button");
            const date = document.createElement("time");
            const label = document.createElement("span");
            button.type = "button";
            button.tabIndex = -1;
            button.hidden = true;
            button.className = "memory-timeline-node";
            button.title = memory.title;
            date.className = "memory-timeline-date";
            date.dateTime = memory.date;
            date.textContent = dateFormat.format(new Date(memory.date));
            label.className = "memory-timeline-label";
            label.textContent = memory.title;
            button.append(date, label);
            button.addEventListener("click", () => {
                centerTimeline(memory.id);
                showDetails(memory.id, button);
            });
            item.append(button);
            items.append(item);
            timelineButtons.set(memory.id, button);
        }

        strand.append(items);
        selectMemory(selectedId);
    }

    function hitTimeline(event) {
        const bounds = timelineCanvas.getBoundingClientRect();
        const x = event.clientX - bounds.left;
        const y = event.clientY - bounds.top;
        let hit = -1;
        let closest = Infinity;

        for (let index = 0; index < timelineSlices.length; index++) {
            const slice = timelineSlices[index];
            let inside = false;
            let centerX = 0;
            let centerY = 0;

            for (let side = 0; side < slice.length; side++) {
                const a = slice[side];
                const b = slice[(side + 1) % slice.length];
                centerX += a.x;
                centerY += a.y;

                if ((a.y > y) !== (b.y > y) && x < a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y)) {
                    inside = !inside;
                }
            }

            const distance = Math.hypot(x - centerX / slice.length, y - centerY / slice.length);

            if ((inside || distance < 16) && distance < closest) {
                hit = index;
                closest = distance;
            }
        }

        if (hit < 0) {
            return undefined;
        }

        const offset = (hit / timelineSliceCount - timelineFocus) * timelineWidth + timeline.scrollLeft;
        const id = Math.min(memories.length, Math.max(1, Math.round(offset / timelinePitch) + 1));
        return timelineButtons.get(id);
    }

    function setTimelineHover(button) {
        if (timelineHover !== button) {
            timelineHover = button;
            timeline.toggleAttribute("data-hovering", Boolean(button));
            refreshTimeline();
        }
    }

    timeline.addEventListener("pointermove", (event) => {
        if (event.pointerType === "mouse" || event.pointerType === "pen") {
            setTimelineHover(hitTimeline(event));
        }
    });
    timeline.addEventListener("pointerleave", () => setTimelineHover(undefined));
    timeline.addEventListener("click", (event) => {
        if (event.detail > 0 && !event.target.closest(".memory-timeline-node")) {
            hitTimeline(event)?.click();
        }
    });

    timeline.addEventListener("keydown", (event) => {
        if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") {
            return;
        }

        event.preventDefault();
        const index = selectedId - 1;
        const step = event.key === "ArrowLeft" ? -1 : 1;
        const memory = memories[index + step];

        if (memory) {
            selectMemory(memory.id);
            centerTimeline(memory.id);
            timelineButtons.get(memory.id).focus({ preventScroll: true });
        }
    });

    timeline.addEventListener("wheel", (event) => {
        if (event.shiftKey || Math.abs(event.deltaY) <= Math.abs(event.deltaX)) {
            return;
        }

        event.preventDefault();
        const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? timeline.clientWidth : 1;
        timeline.scrollLeft += event.deltaY * unit;
    }, { passive: false });

    timelineToggle.addEventListener("click", () => setTopology("timeline"));
    graphToggle.addEventListener("click", () => setTopology("graph"));
    detailClose.addEventListener("click", () => dismissDetails());
    expand.addEventListener("click", () => expandGraph(selectedId));
    timeline.addEventListener("scroll", () => {
        if (!view.hidden && !timelineStage.hidden) {
            positionTimelineTiles(true);
            positionDetails();
        }
    });
    graph.addEventListener("scroll", positionDetails);
    window.addEventListener("resize", () => {
        if (!view.hidden && !timelineStage.hidden) {
            measureTimeline();
            centerTimeline(selectedId);
        }

        positionDetails();
    });
    reducedMotion.addEventListener("change", () => {
        moveGraphNodes();
        refreshTimeline();
    });

    return {
        open() {
            view.hidden = false;

            if (!document.querySelector("#memory-horizon-points").childElementCount) {
                drawHorizonPoints();
            }

            if (!timelineButtons.size) {
                buildTimeline();
            }

            setTopology("timeline");
            timelineButtons.get(selectedId).focus({ preventScroll: true });
        },
        close() {
            dismissDetails(false);
            clearGraphFocus();
            stopTimeline();
            view.hidden = true;
        },
        dismissDetails,
    };
})();
