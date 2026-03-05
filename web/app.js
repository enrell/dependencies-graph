(async function () {
    const res = await fetch("/api/graph");
    const data = await res.json();

    document.getElementById("stats").innerHTML = `
        <div class="stat-badge packages"><span class="dot"></span><span class="value">${data.nodes.length}</span></div>
        <div class="stat-badge links"><span class="dot"></span><span class="value">${data.edges.length}</span></div>
    `;

    const elements = [];

    data.nodes.forEach((n) => {
        elements.push({
            group: "nodes",
            data: { id: n.id, label: n.label, version: n.version, is_root: n.is_root, depth: n.depth },
        });
    });

    data.edges.forEach((e) => {
        elements.push({
            group: "edges",
            data: { id: e.source + "->" + e.target, source: e.source, target: e.target },
        });
    });

    const cy = cytoscape({
        container: document.getElementById("graph"),
        elements,
        style: [
            {
                selector: "node",
                style: {
                    label: "data(label)",
                    shape: "round-rectangle",
                    width: "label",
                    height: "label",
                    padding: "10px",
                    "background-color": "#18181b",
                    "border-color": "#3f3f46",
                    "border-width": 1,
                    color: "#a1a1aa",
                    "font-size": "10px",
                    "font-family": "Inter, sans-serif",
                    "text-valign": "center",
                    "text-halign": "center",
                    "transition-property": "border-color, background-color, color",
                    "transition-duration": "0.15s",
                },
            },
            {
                selector: "node[?is_root]",
                style: {
                    "background-color": "#fafafa",
                    "border-color": "#fafafa",
                    color: "#000000",
                    "font-weight": "600",
                },
            },
            {
                selector: "edge",
                style: {
                    width: 1,
                    "line-color": "#27272a",
                    "target-arrow-color": "#3f3f46",
                    "target-arrow-shape": "triangle",
                    "curve-style": "bezier",
                    opacity: 0.8,
                    "arrow-scale": 0.8,
                    "transition-property": "line-color, target-arrow-color, opacity, width",
                    "transition-duration": "0.15s",
                },
            },
            {
                selector: "node.highlighted",
                style: {
                    "border-color": "#fafafa",
                    "border-width": 2,
                    "background-color": "#27272a",
                    color: "#fafafa"
                },
            },
            {
                selector: "node.faded",
                style: { opacity: 0.1 },
            },
            {
                selector: "edge.highlighted",
                style: {
                    "line-color": "#a1a1aa",
                    "target-arrow-color": "#fafafa",
                    opacity: 1,
                    width: 1.5,
                },
            },
            {
                selector: "edge.faded",
                style: { opacity: 0.05 },
            },
            {
                selector: "node.search-match",
                style: {
                    "border-color": "#8b5cf6",
                    "border-width": 2,
                    "background-color": "#18181b",
                    color: "#fafafa"
                },
            },
        ],
        layout: {
            name: "dagre",
            animate: true,
            animationDuration: 800,
            nodeSep: 40,
            rankSep: 80,
            rankDir: "LR",
            padding: 40,
        },
        minZoom: 0.1,
        maxZoom: 4,
    });

    cy.on("layoutstop", () => {
        document.getElementById("loading").classList.add("done");
    });

    const panel = document.getElementById("detail-panel");
    const panelContent = document.getElementById("panel-content");

    cy.on("tap", "node", (evt) => {
        const node = evt.target;
        const d = node.data();

        const deps = node.outgoers("node");
        const dependents = node.incomers("node");

        let html = `<div class="panel-title">${d.label}${d.is_root ? '<span class="root-badge">root</span>' : ""}</div>`;
        html += `<div class="panel-version">v${d.version}</div>`;

        html += `<div class="panel-meta">`;
        html += `<div class="meta-item"><div class="meta-label">Depth</div><div class="meta-value">${d.depth}</div></div>`;
        html += `<div class="meta-item"><div class="meta-label">Deps</div><div class="meta-value">${deps.length}</div></div>`;
        html += `</div>`;

        if (deps.length > 0) {
            html += `<div class="panel-section"><div class="panel-section-title">Dependencies (${deps.length})</div><ul class="dep-list">`;
            deps.forEach((dep) => {
                html += `<li class="dep-item" data-id="${dep.data("id")}">${dep.data("label")} <span style="opacity:.5">v${dep.data("version")}</span></li>`;
            });
            html += `</ul></div>`;
        }

        if (dependents.length > 0) {
            html += `<div class="panel-section"><div class="panel-section-title">Dependents (${dependents.length})</div><ul class="dep-list">`;
            dependents.forEach((dep) => {
                html += `<li class="dep-item" data-id="${dep.data("id")}">${dep.data("label")} <span style="opacity:.5">v${dep.data("version")}</span></li>`;
            });
            html += `</ul></div>`;
        }

        panelContent.innerHTML = html;
        panel.classList.remove("hidden");

        highlightNeighbors(node);

        panelContent.querySelectorAll(".dep-item").forEach((el) => {
            el.addEventListener("click", () => {
                const target = cy.getElementById(el.dataset.id);
                if (target.length) {
                    cy.animate({ center: { eles: target }, duration: 300 });
                    target.emit("tap");
                }
            });
        });
    });

    cy.on("tap", (evt) => {
        if (evt.target === cy) {
            panel.classList.add("hidden");
            clearHighlights();
        }
    });

    document.getElementById("panel-close").addEventListener("click", () => {
        panel.classList.add("hidden");
        clearHighlights();
    });

    function highlightNeighbors(node) {
        clearHighlights();
        const neighborhood = node.closedNeighborhood();
        cy.elements().not(neighborhood).addClass("faded");
        neighborhood.edges().addClass("highlighted");
        node.addClass("highlighted");
    }

    function clearHighlights() {
        cy.elements().removeClass("faded highlighted search-match");
    }

    const searchInput = document.getElementById("search");
    searchInput.addEventListener("input", () => {
        const q = searchInput.value.trim().toLowerCase();
        clearHighlights();

        if (!q) return;

        const matches = cy.nodes().filter((n) => n.data("label").toLowerCase().includes(q));

        if (matches.length > 0) {
            cy.elements().not(matches).addClass("faded");
            matches.addClass("search-match");

            if (matches.length === 1) {
                cy.animate({ center: { eles: matches }, duration: 300 });
            }
        }
    });

    document.getElementById("btn-fit").addEventListener("click", () => cy.fit(undefined, 40));
    document.getElementById("btn-zoom-in").addEventListener("click", () => cy.zoom({ level: cy.zoom() * 1.3, renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 } }));
    document.getElementById("btn-zoom-out").addEventListener("click", () => cy.zoom({ level: cy.zoom() / 1.3, renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 } }));
})();
