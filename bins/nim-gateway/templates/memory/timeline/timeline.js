customElements.define("nim-memory-timeline", class extends HTMLElement {
    timelinePoint(progress, width, height) {
        const wave = Math.sin(progress * Math.PI * 4.2);
        return { x: (progress - 0.5) * width, y: height * (0.59 + wave * 0.035) - height / 2, z: 0 };
    }

    projectTimeline({ x, y, z }, width, height) {
        const scale = width / (width + z);
        return { x: width / 2 + x * scale, y: height / 2 + y * scale };
    }

    timelineSlice(progress, width, height, time, lift) {
        const point = this.timelinePoint(progress, width, height);
        const next = this.timelinePoint(progress + 0.001, width, height);
        const dx = next.x - point.x;
        const dy = next.y - point.y;
        const dz = next.z - point.z;
        const length = Math.hypot(dx, dy, dz);
        const planar = Math.hypot(dx, dy);
        const nx = -dy / planar;
        const ny = dx / planar;
        const radius = Math.min(width * 0.10, height * 0.14) * (1 + lift * 0.22);
        const twist = progress * 7.1 + time * 0.21;
        const corners = [];

        for (let side = 0; side < 6; side++) {
            const angle = side * Math.PI / 3 + twist;
            const a = Math.cos(angle) * radius;
            const b = Math.sin(angle) * radius * 0.76;
            corners.push(this.projectTimeline({
                x: point.x + nx * a - dz * ny / length * b - nx * lift * 23,
                y: point.y + ny * a + dz * nx / length * b - ny * lift * 23,
                z: point.z + planar / length * b - lift * width * 0.035,
            }, width, height));
        }

        return corners;
    }

    traceTimeline(timelineContext, points) {
        timelineContext.moveTo(points[0].x, points[0].y);

        for (let index = 1; index < points.length; index++) {
            timelineContext.lineTo(points[index].x, points[index].y);
        }
    }
});