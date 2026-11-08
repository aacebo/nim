const canvas = document.querySelector("#orb");
const context = canvas.getContext("2d");

if (context) {
    animateOrb();
}

function animateOrb() {
    const turn = 2 * Math.PI;
    const latitudeSteps = 38;
    const longitudeSteps = 72;
    const tilt = 0.34;
    const roll = -0.38;
    const tiltCos = Math.cos(tilt);
    const tiltSin = Math.sin(tilt);
    const rollCos = Math.cos(roll);
    const rollSin = Math.sin(roll);
    const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)");
    const beads = Array.from({ length: 760 }, (_, index) => ({
        latitude: Math.asin(2 * Math.random() - 1),
        longitude: Math.random() * turn,
        phase: Math.random() * turn,
        speed: 0.16 + Math.random() * 0.29,
        size: 0.65 + Math.random() * 0.8,
        distance: 1.05 + Math.random() * 0.16,
        halo: index % 3 === 0,
    }));
    const mesh = Array.from({ length: latitudeSteps + 1 }, (_, row) =>
        Array.from({ length: longitudeSteps }, (_, column) =>
            direction(Math.PI * (row / latitudeSteps - 0.5), turn * column / longitudeSteps),
        ),
    );
    const points = mesh.map((ring) => ring.map(() => ({ x: 0, y: 0, z: 0 })));
    const beadPoint = { x: 0, y: 0, z: 0 };
    let size;
    let radius;
    let color;
    let frame;
    let previous;
    let time = 0;

    function direction(latitude, longitude) {
        const ring = Math.cos(latitude);
        const x = ring * Math.sin(longitude);
        const y = Math.sin(latitude);
        const z = ring * Math.cos(longitude);
        const tiltedY = y * tiltCos - z * tiltSin;

        return {
            x: x * rollCos - tiltedY * rollSin,
            y: x * rollSin + tiltedY * rollCos,
            z: y * tiltSin + z * tiltCos,
        };
    }

    function project(direction, point) {
        const { x, y, z } = direction;
        const swell = 1
            + 0.071 * Math.sin(5.2 * x + 3.8 * y + 2.4 * z + time * 0.44)
            + 0.049 * Math.sin(7.1 * y - 4.5 * x + 3.7 * z - time * 0.31)
            + 0.027 * Math.sin(11 * z + 3.6 * y + time * 0.38);
        const scale = radius * swell * (1 + 0.11 * z);

        point.x = x * scale;
        point.y = y * scale;
        point.z = z;
    }

    function drawMesh() {
        const near = new Path2D();
        const far = new Path2D();

        for (let row = 0; row <= latitudeSteps; row++) {
            for (let column = 0; column < longitudeSteps; column++) {
                project(mesh[row][column], points[row][column]);
            }
        }

        function segment(start, end) {
            const path = start.z + end.z >= 0 ? near : far;
            path.moveTo(start.x, start.y);
            path.lineTo(end.x, end.y);
        }

        for (let row = 0; row <= latitudeSteps; row++) {
            for (let column = 0; column < longitudeSteps; column++) {
                const point = points[row][column];

                if (row > 0 && row < latitudeSteps) {
                    segment(point, points[row][(column + 1) % longitudeSteps]);
                }

                if (row < latitudeSteps) {
                    segment(point, points[row + 1][column]);
                }
            }
        }

        context.strokeStyle = color;
        context.lineWidth = 0.55;
        context.globalAlpha = 0.21;
        context.stroke(far);
        context.lineWidth = 3.6;
        context.globalAlpha = 0.14;
        context.stroke(near);
        context.lineWidth = 0.75;
        context.globalAlpha = 0.87;
        context.stroke(near);
    }

    function drawBeads() {
        const count = size < 500 ? 440 : beads.length;
        context.fillStyle = color;

        for (let index = 0; index < count; index++) {
            const bead = beads[index];
            const phase = bead.phase + time * bead.speed;
            const pulse = 0.5 + 0.5 * Math.sin(phase * 1.4);
            const longitude = bead.longitude + 0.038 * Math.sin(phase);

            if (bead.halo) {
                const distance = radius * (bead.distance + 0.027 * Math.sin(phase));
                beadPoint.x = Math.cos(longitude) * distance;
                beadPoint.y = Math.sin(longitude) * distance;
                beadPoint.z = 0;
            } else {
                const latitude = bead.latitude + 0.026 * Math.cos(phase);
                project(direction(latitude, longitude), beadPoint);
            }

            const light = beadPoint.z < 0 ? 0.33 : 1;
            const dot = bead.size * (0.82 + 0.28 * pulse) * Math.max(0.67, size / 760);
            context.globalAlpha = light * (0.44 + 0.52 * pulse);
            context.beginPath();
            context.arc(beadPoint.x, beadPoint.y, dot, 0, turn);
            context.fill();
        }
    }

    function draw() {
        context.clearRect(-size / 2, -size / 2, size, size);
        context.globalCompositeOperation = "lighter";
        drawMesh();
        drawBeads();
        context.globalAlpha = 1;
    }

    function resize() {
        size = canvas.getBoundingClientRect().width;
        const pixels = Math.round(size * Math.min(devicePixelRatio, 2));
        canvas.width = canvas.height = pixels;
        radius = size * 0.395;
        const scale = pixels / size;
        context.setTransform(scale, 0, 0, scale, pixels / 2, pixels / 2);
        color = context.createLinearGradient(-radius * 0.83, -radius, radius * 0.65, radius);
        color.addColorStop(0, "#00e7ff");
        color.addColorStop(0.34, "#4672ff");
        color.addColorStop(0.62, "#9c57ff");
        color.addColorStop(0.86, "#f02eff");
        color.addColorStop(1, "#ff58c8");
        draw();
    }

    function tick(now) {
        if (previous !== undefined) {
            time += Math.min(now - previous, 50) / 1000;
        }

        previous = now;
        draw();
        frame = requestAnimationFrame(tick);
    }

    function syncMotion() {
        cancelAnimationFrame(frame);
        previous = undefined;

        if (!document.hidden && !reducedMotion.matches) {
            frame = requestAnimationFrame(tick);
        }
    }

    resize();
    syncMotion();
    window.addEventListener("resize", resize);
    document.addEventListener("visibilitychange", syncMotion);
    reducedMotion.addEventListener("change", syncMotion);
}
