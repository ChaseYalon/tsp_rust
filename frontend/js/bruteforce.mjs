class Point{
    x;
    y;
    constructor(x, y){
        this.x = x;
        this.y = y;
    }
}
class Node{
    pt;
    cities;
    /**
     * 
     * @param {Point} pt - Base point that the node represents
     * @param {Node[]} cities - Nodes that the city could travel to
     */
    constructor(pt, cities){
        this.pt = pt;
        this.cities = cities;
    }

}

/**
 * 
 * @param {Point[]} points 
 * @returns {Point[]} solved array of points
 * @description branch and bound TSP solver
 */
export function bruteForce(points) {
    const n = points.length;

    // Precompute distance matrix
    const dist = Array.from({ length: n }, () => Array(n).fill(0));
    for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
            const dx = points[i].x - points[j].x;
            const dy = points[i].y - points[j].y;
            dist[i][j] = Math.hypot(dx, dy);
        }
    }

    let bestPath = [];
    let bestCost = Infinity;

    /**
     * Lower bound estimate for a partial path
     * Here: current path cost + sum of cheapest edge for each unvisited city
     */
    function bound(path, cost) {
        const remaining = [];
        for (let i = 0; i < n; i++) {
            if (!path.includes(i)) remaining.push(i);
        }
        let minExtra = 0;
        for (const city of remaining) {
            let bestEdge = Infinity;
            for (let other = 0; other < n; other++) {
                if (city !== other && !path.includes(other)) {
                    bestEdge = Math.min(bestEdge, dist[city][other]);
                }
            }
            minExtra += bestEdge;
        }
        return cost + minExtra;
    }

    /**
     * Depth-first search with branch-and-bound pruning
     * @param {number[]} path
     * @param {number} cost
     */
    function dfs(path, cost) {
        if (path.length === n) {
            const total = cost + dist[path[n - 1]][path[0]]; // return to start
            if (total < bestCost) {
                bestCost = total;
                bestPath = [...path];
            }
            return;
        }

        if (bound(path, cost) >= bestCost) return; // prune branch

        for (let next = 0; next < n; next++) {
            if (!path.includes(next)) {
                dfs([...path, next], cost + dist[path[path.length - 1]][next]);
            }
        }
    }

    dfs([0], 0); // start at first point

    return bestPath.map(i => points[i]);
}
export class BFManager {
    constructor(button) {
        this.button = button;
        this.enabled = true;
    }
    
    get isEnabled() {
        return this.enabled;
    }
    
    enable() {
        this.enabled = true;
        this.button.classList.remove('bf-off');
        this.button.classList.add('bf-but');
        this.button.innerHTML = "Brute Force Enabled";
        this.checkPointLimit()
    }
    
    disable() {
        this.enabled = false;
        this.button.classList.remove('bf-but');
        this.button.classList.add('bf-off');
        this.button.innerHTML = "Brute Force Disabled";
    }
    
    toggle() {
        if (this.enabled) {
            this.disable();
        } else {
            this.enable();
        }
    }
    
    // Disable if too many points (>15)
    checkPointLimit(pointCount) {
        if (pointCount > 15 && this.enabled) {
            this.disable();
        }
    }
}
