import  util  from "./util.mjs";
import { brute } from "./brute.mjs";


export const christophides = {
    // Helper function to find odd degree vertices in the MST
    findOddDegreeVertices(mst, points) {
        const degree = Array(points.length).fill(0);
        for (const [u, v] of mst) {
            degree[u]++;
            degree[v]++;
        }
        return points.filter((_, index) => degree[index] % 2 === 1);
    },

    // Function to find the minimum weight matching of odd degree vertices
    minimumWeightMatching(oddVertices) {
        const edges = [];
        const used = new Set();
        const matching = [];

        // Generate all pairwise edges with their weights
        for (let i = 0; i < oddVertices.length; i++) {
            for (let j = i + 1; j < oddVertices.length; j++) {
                const weight = util.calcDist(oddVertices[i], oddVertices[j]);
                edges.push([i, j, weight]);
            }
        }

        // Sort edges by weight
        edges.sort((a, b) => a[2] - b[2]);

        // Perform greedy matching
        for (const [i, j] of edges) {
            if (!used.has(i) && !used.has(j)) {
                matching.push([oddVertices[i], oddVertices[j]]);
                used.add(i);
                used.add(j);
            }
        }

        return matching;
    },

    // Function to generate Eulerian circuit
    eulerianCircuit(mst, matching) {
        const graph = new Map();

        // Add MST edges to the graph
        for (const [u, v] of mst) {
            if (!graph.has(u)) graph.set(u, []);
            if (!graph.has(v)) graph.set(v, []);
            graph.get(u).push(v);
            graph.get(v).push(u);
        }

        // Add matching edges to the graph
        for (const [u, v] of matching) {
            if (!graph.has(u)) graph.set(u, []);
            if (!graph.has(v)) graph.set(v, []);
            graph.get(u).push(v);
            graph.get(v).push(u);
        }

        // Hierholzer's Algorithm for Eulerian Circuit
        const stack = [];
        const circuit = [];
        const start = Array.from(graph.keys())[0];
        stack.push(start);

        while (stack.length > 0) {
            const v = stack[stack.length - 1];
            if (graph.get(v).length > 0) {
                const u = graph.get(v).pop();
                graph.get(u).splice(graph.get(u).indexOf(v), 1);
                stack.push(u);
            } else {
                circuit.push(stack.pop());
            }
        }

        return circuit.reverse();
    },

    // Function to convert Eulerian circuit to Hamiltonian circuit
    eulerianToHamiltonian(eulerianCircuit) {
        const visited = new Set();
        const hamiltonianCircuit = [];

        for (const vertex of eulerianCircuit) {
            if (!visited.has(vertex)) {
                visited.add(vertex);
                hamiltonianCircuit.push(vertex);
            }
        }

        // Close the circuit
        hamiltonianCircuit.push(hamiltonianCircuit[0]);
        return hamiltonianCircuit;
    },

    // Main Christofides algorithm
    christofidesTSP(points) {
        // Step 1: Compute MST
        const mstEdges = brute.mst(points);

        // Step 2: Find odd degree vertices
        const oddVertices = this.findOddDegreeVertices(mstEdges, points);

        // Step 3: Compute minimum weight perfect matching
        const matching = this.minimumWeightMatching(oddVertices);

        // Step 4: Combine MST and matching to create Eulerian graph
        const eulerianPath = this.eulerianCircuit(mstEdges, matching);

        // Step 5: Generate Hamiltonian circuit
        const hamiltonianPath = this.eulerianToHamiltonian(eulerianPath.map(v => points[v]));

        return hamiltonianPath;
    }
};