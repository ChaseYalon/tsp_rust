/**
 * @param {Point[]} points - Array of points to visit
 * @returns {Point[]} - Array representing the tour using nearest neighbor
 */
export function nearestNeighbor(points) {
    if (points.length === 0) return [];

    const unvisited = points.slice(); // copy of points
    const tour = [];
    
    // Start from the first point
    let current = unvisited.shift();
    tour.push(current);

    while (unvisited.length > 0) {
        // Find the nearest unvisited point
        let nearestIndex = 0;
        let nearestDist = calcDist(current, unvisited[0]);
        for (let i = 1; i < unvisited.length; i++) {
            const d = calcDist(current, unvisited[i]);
            if (d < nearestDist) {
                nearestDist = d;
                nearestIndex = i;
            }
        }

        // Move to the nearest point
        current = unvisited.splice(nearestIndex, 1)[0];
        tour.push(current);
    }
    return tour;
}

/**
 * 
 * @param {Point[]} points - Input points
 * @returns {Edge[]} - An array of edges with from and to properties
 */
export function computeMST(points) {
    const n = points.length;
    if (n === 0) return [];

    const visited = new Array(n).fill(false);
    const key = new Array(n).fill(Infinity);
    const parent = new Array(n).fill(-1);

    key[0] = 0;

    for (let i = 0; i < n; i++) {
        // Find minimum key vertex not yet visited
        let u = -1;
        let minKey = Infinity;
        for (let v = 0; v < n; v++) {
            if (!visited[v] && key[v] < minKey) {
                minKey = key[v];
                u = v;
            }
        }

        visited[u] = true;

        // Update keys of adjacent vertices
        for (let v = 0; v < n; v++) {
            const d = calcDist(points[u], points[v]);
            if (!visited[v] && d < key[v]) {
                key[v] = d;
                parent[v] = u;
            }
        }
    }

    // Build edge list with actual points
    const edges = [];
    for (let v = 1; v < n; v++) {
        edges.push({ from: points[parent[v]], to: points[v] });
    }
    return edges;
}

function minWeightMatching(points, mstEdges) {
    const n = points.length;
    const degree = new Array(n).fill(0);

    mstEdges.forEach(e => {
        degree[pointIndex(points, e.from)]++;
        degree[pointIndex(points, e.to)]++;
    });

    const oddVertices = [];
    for (let i = 0; i < n; i++) {
        if (degree[i] % 2 === 1) oddVertices.push(points[i]);
    }

    const matching = [];
    const used = new Set();

    while (oddVertices.length > 0) {
        let u = oddVertices.pop();
        if (used.has(u)) continue;

        let minDist = Infinity;
        let closest = null;
        for (let v of oddVertices) {
            if (!used.has(v)) {
                const d = calcDist(u, v);
                if (d < minDist) {
                    minDist = d;
                    closest = v;
                }
            }
        }
        if (closest) {
            matching.push({ from: u, to: closest });
            used.add(u);
            used.add(closest);
            // Remove closest from oddVertices array
            const closestIndex = oddVertices.indexOf(closest);
            if (closestIndex !== -1) {
                oddVertices.splice(closestIndex, 1);
            }
        }
    }

    return matching;
}

/**
 * Build Eulerian multigraph adjacency list
 */
function buildAdjacencyList(points, edges) {
    const adj = Array.from({ length: points.length }, () => []);
    edges.forEach(e => {
        const fromIdx = pointIndex(points, e.from);
        const toIdx = pointIndex(points, e.to);
        adj[fromIdx].push(toIdx);
        adj[toIdx].push(fromIdx);
    });
    return adj;
}

function pointIndex(points, p) {
    for(let i = 0; i < points.length; i++){
        if (points[i] === p){ // Use strict equality
            return i;
        }
    }
    return -1;
}

/**
 * Find Eulerian tour using Hierholzer's algorithm
 */
function findEulerianTour(adj) {
    const tour = [];
    const stack = [0];
    const localAdj = adj.map(a => a.slice()); // copy

    while (stack.length > 0) {
        const v = stack[stack.length - 1];
        if (localAdj[v].length === 0) {
            tour.push(v);
            stack.pop();
        } else {
            const u = localAdj[v].pop();
            const index = localAdj[u].indexOf(v);
            localAdj[u].splice(index, 1);
            stack.push(u);
        }
    }

    return tour;
}

/**
 * Christofides TSP
 * @param {Point[]} points
 * @returns {Point[]} - Hamiltonian tour
 */
export function christofidesAlgo(points) {
    const n = points.length;
    if (n === 0) return [];

    // Step 1: MST
    const mst = computeMST(points);

    // Step 2: Greedy min-weight matching on odd-degree vertices
    const matching = minWeightMatching(points, mst);

    // Step 3: Combine MST + matching edges
    const allEdges = mst.concat(matching);

    // Step 4: Build adjacency list - FIXED: pass points instead of n
    const adj = buildAdjacencyList(points, allEdges);

    // Step 5: Eulerian tour
    const eulerTour = findEulerianTour(adj);

    // Step 6: Shortcut repeated vertices to get Hamiltonian tour
    const visited = new Set();
    const tour = [];
    for (let idx of eulerTour) {
        if (!visited.has(idx)) {
            tour.push(points[idx]);
            visited.add(idx);
        }
    }

    return tour;
}

function calcDist(a, b){
    return Math.sqrt(((a.x - b.x) * (a.x - b.x)) + ((a.y - b.y) * (a.y - b.y)));
}