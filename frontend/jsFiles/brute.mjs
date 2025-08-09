import  util  from "./util.mjs";
import { global } from "./globals.mjs";


export const brute = {
    permute(arr) {
        if (arr.length === 1) return [arr];
        const results = [];
        for (let i = 0; i < arr.length; i++) {
            const rest = arr.slice(0, i).concat(arr.slice(i + 1));
            const restPermutations =  brute.permute(rest);
            for (let perm of restPermutations) {
                results.push([arr[i], ...perm]);
            }
        }
        return results;
    },
    tspBruteForce(cities) {
        const permutations = brute.permute(cities); // Use brute.permute here
        let shortestDistance = Infinity;
        let bestRoute = [];

        // Try all permutations and find the shortest path
        for (let route of permutations) {
            const distance = util.pathDist(route);
            if (distance < shortestDistance) {
                shortestDistance = distance;
                bestRoute = route;
            }
        }

        return { bestRoute, shortestDistance };
    },
    //Khursals Algorithm
    mst(points) {
        const edges = [];
        const n = points.length; // Fix: Get length instead of treating points as a number
    
        // Generate all edges with their weights
        for (let i = 0; i < n; i++) {
            for (let j = i + 1; j < n; j++) {
                const weight = util.calcDist(points[i], points[j]); // Fix: Use points[j]
                edges.push([i, j, weight]);
            }
        }
    
        // Sort edges by weight in ascending order
        edges.sort((a, b) => a[2] - b[2]);
    
        // Union-Find setup
        const parent = Array.from({ length: n }, (_, i) => i);
        const rank = Array(n).fill(0);
    
        function find(node) {
            if (parent[node] !== node) {
                parent[node] = find(parent[node]); // Path compression
            }
            return parent[node];
        }
    
        function union(node1, node2) {
            const root1 = find(node1);
            const root2 = find(node2);
    
            if (root1 !== root2) {
                if (rank[root1] > rank[root2]) {
                    parent[root2] = root1;
                } else if (rank[root1] < rank[root2]) {
                    parent[root1] = root2;
                } else {
                    parent[root2] = root1;
                    rank[root1]++;
                }
                return true; // Successful union
            }
            return false; // Cycle detected
        }
    
        // Build the minimum spanning tree
        const maxTreeEdges = [];
        for (const [u, v, weight] of edges) {
            if (union(u, v)) {
                maxTreeEdges.push([u, v, weight]);
                if (maxTreeEdges.length === n - 1) break; // Tree complete
            }
        }
    
        let newV = null;
        let newU = null;
        let newW = -Infinity; // Start with the smallest possible value to find the largest edge
    
        for (let i = 0; i < points.length; i++) {
            for (let j = 0; j < points.length; j++) {
                if (i === j) continue; // Skip self-loops
    
                const dist = util.calcDist(points[i], points[j]); // Fix: Use correct points array
    
                // Check if the edge is valid and has the largest weight
                if (dist > newW && !edgeExists(maxTreeEdges, i, j)) {
                    newV = i;
                    newU = j;
                    newW = dist;
                }
            }
        }
    
        // If no valid edge is found, return an empty array instead of null
        if (newW === -Infinity) {
            console.log("No valid edge found.");
            return []; // Fix: Return empty array to prevent TypeError
        }
    
        // Add the new edge to maxTreeEdges
        //It kept screwing up
        //maxTreeEdges.push([newU, newV, newW]);
    
        return maxTreeEdges;
    
        // Helper function to check if an edge already exists in maxTreeEdges
        function edgeExists(edges, u, v) {
            return edges.some(edge => (edge[0] === u && edge[1] === v) || (edge[0] === v && edge[1] === u));
        }
    }
     

}