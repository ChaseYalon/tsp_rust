import util from "./util.mjs";

export const nearest = {
    neighbor(inPoints) {
        let points = [...inPoints]; // Create a copy to avoid mutating original array
        let tour = []; // Store ordered path

        if (points.length === 0) return tour; // Return empty array if no points exist

        // Start at the first point
        let current = points.shift();
        tour.push(current);

        while (points.length > 0) {
            let bestDist = Infinity;
            let bestIndex = -1;

            // Find the closest point
            for (let i = 0; i < points.length; i++) {
                let currDist = util.calcDist(current, points[i]);
                if (currDist < bestDist) {
                    bestDist = currDist;
                    bestIndex = i;
                }
            }

            // Move the closest point to the tour
            current = points.splice(bestIndex, 1)[0];
            tour.push(current);
        }

        return tour;
    }
};

