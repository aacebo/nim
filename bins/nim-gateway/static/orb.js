const canvas = document.querySelector("#orb");
const context = canvas.getContext("2d");
const trigger = document.querySelector("#orb-trigger");

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
    const gradientStops = [0, 0.34, 0.62, 0.86, 1];
    const palettes = [
        [[0, 231, 255], [70, 114, 255], [156, 87, 255], [240, 46, 255], [255, 88, 200]],
        [[103, 107, 255], [129, 121, 255], [189, 143, 255], [247, 105, 203], [255, 118, 155]],
        [[18, 245, 182], [48, 200, 227], [64, 139, 255], [127, 87, 255], [191, 104, 255]],
    ];
    const transitionDuration = 18;
    const rippleDuration = 1.5;
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
    let ripple;
    let rippleFront;
    let rippleStrength;
    let rippleGlow;

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
        applyRipple(point);
    }

    function applyRipple(point, amplitude = 1) {
        point.glow = 0;

        if (!ripple) {
            return;
        }

        const dx = point.x / radius - ripple.x;
        const dy = point.y / radius - ripple.y;
        const distance = Math.hypot(dx, dy);
        const wave = (distance - rippleFront) / 0.21;
        const band = Math.exp(-wave * wave);
        const displacement = amplitude * rippleStrength * Math.cos(wave * 1.7) * band;
        const scale = displacement / Math.max(distance, 0.09);

        point.x += dx * scale;
        point.y += dy * scale;
        point.glow = band;
    }

    function startRipple(event) {
        if (reducedMotion.matches || document.hidden) {
            return;
        }

        const bounds = canvas.getBoundingClientRect();
        const pointer = event.detail > 0;
        ripple = {
            x: pointer ? (event.clientX - bounds.left - bounds.width / 2) / radius : 0,
            y: pointer ? (event.clientY - bounds.top - bounds.height / 2) / radius : 0,
            start: time,
        };
    }

    function gradient() {
        const position = time / transitionDuration;
        const index = Math.floor(position);
        const progress = position - index;
        const blend = progress * progress * (3 - 2 * progress);
        const from = palettes[index % palettes.length];
        const to = palettes[(index + 1) % palettes.length];
        const result = context.createLinearGradient(-radius * 0.83, -radius, radius * 0.65, radius);

        for (let stop = 0; stop < gradientStops.length; stop++) {
            const channels = from[stop].map((channel, i) => channel + (to[stop][i] - channel) * blend);
            result.addColorStop(gradientStops[stop], `rgb(${channels.join(", ")})`);
        }

        return result;
    }

    function drawMesh() {
        const near = new Path2D();
        const far = new Path2D();
        const crest = ripple ? new Path2D() : undefined;

        for (let row = 0; row <= latitudeSteps; row++) {
            for (let column = 0; column < longitudeSteps; column++) {
                project(mesh[row][column], points[row][column]);
            }
        }

        function segment(start, end) {
            const path = start.z + end.z >= 0 ? near : far;
            path.moveTo(start.x, start.y);
            path.lineTo(end.x, end.y);

            if (crest && path === near && start.glow + end.glow > 1.2) {
                crest.moveTo(start.x, start.y);
                crest.lineTo(end.x, end.y);
            }
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
        context.lineWidth = 2.4;
        context.globalAlpha = 0.11;
        context.stroke(near);
        context.lineWidth = 0.75;
        context.globalAlpha = 0.87;
        context.stroke(near);

        if (crest) {
            context.lineWidth = 3.4;
            context.globalAlpha = rippleGlow * 0.3;
            context.stroke(crest);
            context.lineWidth = 1.6;
            context.globalAlpha = rippleGlow;
            context.stroke(crest);
        }
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
                applyRipple(beadPoint, 0.5);
            } else {
                const latitude = bead.latitude + 0.026 * Math.cos(phase);
                project(direction(latitude, longitude), beadPoint);
            }

            const light = beadPoint.z < 0 ? 0.33 : 1;
            const glow = ripple ? beadPoint.glow * rippleGlow : 0;
            const dot = bead.size * (0.82 + 0.28 * pulse + 0.6 * glow) * Math.max(0.67, size / 760);
            context.globalAlpha = light * Math.min(1, 0.44 + 0.52 * pulse + glow);
            context.beginPath();
            context.arc(beadPoint.x, beadPoint.y, dot, 0, turn);
            context.fill();
        }
    }

    function draw() {
        if (ripple) {
            const progress = (time - ripple.start) / rippleDuration;

            if (progress >= 1) {
                ripple = undefined;
            } else {
                rippleFront = progress * 2.5;
                rippleGlow = Math.min(progress * 18, 1) * (1 - progress);
                rippleStrength = radius * 0.0425 * rippleGlow;
            }
        }

        color = gradient();
        context.clearRect(-size / 2, -size / 2, size, size);
        drawMesh();
        drawBeads();
        context.globalAlpha = 1;
    }

    function resize() {
        const nextSize = canvas.getBoundingClientRect().width;

        if (nextSize <= 0) {
            return;
        }

        const pixels = Math.round(nextSize * Math.min(devicePixelRatio, 2));

        if (size === nextSize && canvas.width === pixels && canvas.height === pixels) {
            return;
        }

        size = nextSize;
        canvas.width = canvas.height = pixels;
        radius = size * 0.38;
        const scale = pixels / size;
        context.setTransform(scale, 0, 0, scale, pixels / 2, pixels / 2);
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

        if (reducedMotion.matches) {
            ripple = undefined;
            draw();
        } else if (!document.hidden) {
            frame = requestAnimationFrame(tick);
        }
    }

    resize();
    syncMotion();
    trigger.addEventListener("click", startRipple);
    // Layout changes can resize the canvas without a window resize event.
    const resizeObserver = new ResizeObserver(resize);
    resizeObserver.observe(canvas);
    window.addEventListener("resize", resize);

    // Re-arm when zoom or moving between displays changes pixel density.
    function watchPixelRatio() {
        const resolution = matchMedia(`(resolution: ${devicePixelRatio}dppx)`);
        resolution.addEventListener("change", () => {
            resize();
            watchPixelRatio();
        }, { once: true });
    }

    watchPixelRatio();
    document.addEventListener("visibilitychange", syncMotion);
    reducedMotion.addEventListener("change", syncMotion);
}
