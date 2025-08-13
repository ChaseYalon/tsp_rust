/**
 * Branch and bound TSP solver
 * @param {Point[]} points - Array of points with x, y properties
 * @returns {Point[]} - Optimal tour as array of points
 */
/**
 * Branch and bound TSP solver
 * @param {Point[]} points - Array of points with x, y properties
 * @returns {{from: Point, to: Point}[]} - Optimal tour as array of edges
 */
export function bruteForce(points) {
    const n = points.length;

    // Handle edge cases
    if (n === 0) return [];
    if (n === 1) return [];
    if (n === 2) {
        return [
            { from: points[0], to: points[1] },
            { from: points[1], to: points[0] }
        ];
    }

    // Precompute distance matrix
    const dist = Array.from({ length: n }, () => Array(n).fill(0));
    for (let i = 0; i < n; i++) {
        for (let j = 0; j < n; j++) {
            if (i === j) {
                dist[i][j] = 0;
            } else {
                const dx = points[i].x - points[j].x;
                const dy = points[i].y - points[j].y;
                dist[i][j] = Math.sqrt(dx * dx + dy * dy);
            }
        }
    }

    let bestPath = null;
    let bestCost = Infinity;

    /**
     * Lower bound calculation using minimum spanning tree approximation
     */
    function calculateLowerBound(visited, currentCost, lastCity) {
        let bound = currentCost;
        const unvisited = [];

        // Collect unvisited cities
        for (let i = 0; i < n; i++) {
            if (!visited[i]) {
                unvisited.push(i);
            }
        }

        if (unvisited.length === 0) {
            // Complete tour - add return cost
            return bound + dist[lastCity][0];
        }

        if (unvisited.length === 1) {
            // One city left - add cost to visit it and return
            const city = unvisited[0];
            return bound + dist[lastCity][city] + dist[city][0];
        }

        // Add cost from last city to cheapest unvisited city
        let minFromLast = Infinity;
        for (const city of unvisited) {
            minFromLast = Math.min(minFromLast, dist[lastCity][city]);
        }
        bound += minFromLast;

        // Add minimum spanning tree cost for remaining cities
        if (unvisited.length > 1) {
            // Simple MST approximation: sum of two smallest edges for each unvisited city
            for (const city of unvisited) {
                const edges = [];

                // Edges to other unvisited cities
                for (const otherCity of unvisited) {
                    if (city !== otherCity) {
                        edges.push(dist[city][otherCity]);
                    }
                }

                // Edge back to start
                edges.push(dist[city][0]);

                // Sort and take smallest edge (conservative bound)
                edges.sort((a, b) => a - b);
                if (edges.length > 0) {
                    bound += edges[0] * 0.5; // Divide by 2 since MST uses each edge once
                }
            }
        }

        return bound;
    }

    /**
     * Recursive branch and bound search
     */
    function branchAndBound(path, visited, currentCost) {
        const currentCity = path[path.length - 1];

        // If we've visited all cities
        if (path.length === n) {
            const totalCost = currentCost + dist[currentCity][0]; // Return to start
            if (totalCost < bestCost) {
                bestCost = totalCost;
                bestPath = [...path];
            }
            return;
        }

        // Calculate lower bound for current state
        const lowerBound = calculateLowerBound(visited, currentCost, currentCity);

        // Prune if bound exceeds current best
        if (lowerBound >= bestCost) {
            return;
        }

        // Try all unvisited cities
        for (let nextCity = 0; nextCity < n; nextCity++) {
            if (!visited[nextCity]) {
                // Mark as visited
                visited[nextCity] = true;
                path.push(nextCity);

                // Recurse
                branchAndBound(
                    path,
                    visited,
                    currentCost + dist[currentCity][nextCity]
                );

                // Backtrack
                path.pop();
                visited[nextCity] = false;
            }
        }
    }

    // Initialize search starting from city 0
    const initialVisited = Array(n).fill(false);
    initialVisited[0] = true;

    branchAndBound([0], initialVisited, 0);

    // Convert indices back to edges with actual point objects
    if (bestPath) {
        const edges = [];
        for (let i = 0; i < bestPath.length; i++) {
            const fromIndex = bestPath[i];
            const toIndex = bestPath[(i + 1) % bestPath.length]; // wrap back to start
            edges.push({
                from: points[fromIndex],
                to: points[toIndex]
            });
        }
        return edges;
    }

    // Fallback: return empty if no solution found
    return [];
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
        this.checkPointLimit();
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

function calculateLowerBound(visited, currentCost, lastCity) {
    let bound = currentCost;
    const unvisited = [];
    
    // Collect unvisited cities
    for (let i = 0; i < n; i++) {
        if (!visited[i]) {
            unvisited.push(i);
        }
    }
    
    if (unvisited.length === 0) {
        // Complete tour - add return cost
        return bound + dist[lastCity][0];
    }
    
    if (unvisited.length === 1) {
        // One city left - add cost to visit it and return
        const city = unvisited[0];
        return bound + dist[lastCity][city] + dist[city][0];
    }
    
    // Add cost from last city to cheapest unvisited city
    let minFromLast = Infinity;
    for (const city of unvisited) {
        minFromLast = Math.min(minFromLast, dist[lastCity][city]);
    }
    bound += minFromLast;
    
    // Add minimum spanning tree cost for remaining cities
    if (unvisited.length > 1) {
        // Simple MST approximation: sum of two smallest edges for each unvisited city
        for (const city of unvisited) {
            const edges = [];
            
            // Edges to other unvisited cities
            for (const otherCity of unvisited) {
                if (city !== otherCity) {
                    edges.push(dist[city][otherCity]);
                }
            }
            
            // Edge back to start
            edges.push(dist[city][0]);
            
            // Sort and take smallest edge (conservative bound)
            edges.sort((a, b) => a - b);
            if (edges.length > 0) {
                bound += edges[0] * 0.5; // Divide by 2 since MST uses each edge once
            }
        }
    }
    
    return bound;
}
