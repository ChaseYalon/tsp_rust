import { canvasEffects } from "./canvasEffects.mjs";
import  util  from "./util.mjs";
import { brute } from "./brute.mjs";
import { christophides } from "./christophides.mjs";
import { global } from "./globals.mjs";


export const tests = {
    async test(mstEnabled, pointCount, testCount) {
        console.log("Running tests...");
        let totalRatio = 0;
        let totalCRatio = 0;
        for (let i = 0; i < testCount; i++) {
            // Initialize canvas and points
            canvasEffects.innit();
            global.arrs.unusedPoints = [];
            global.arrs.points = [];

            // Add random points
            for (let j = 0; j < pointCount; j++) {
                util.addPoint(util.rand(0,500),util.rand(0,500));
            }

            // Compute hull using the iterative algorithm
            let response = await fetch(`/solve?P=${encodeURIComponent(JSON.stringify(global.arrs.points))},P=${encodeURIComponent(JSON.stringify(global.arrs.unusedPoints))}`);
            let hull = response.json;
            let hullDist = util.pathDist(hull);

            if (mstEnabled) {
                // Compute MST and its distance
                let mstEdges = brute.mst(points);
                let mstDist = util.ePathDist(mstEdges);

                // Calculate the ratio
                let ratio = hullDist / mstDist;
                totalRatio += ratio;

                let christhopides =christophides.christofidesTSP(points)
                totalCRatio += (util.pathDist(christhopides)/mstDist);


            } else {
                console.warn("MST testing is disabled.");
                return;
            }
        }

        // Average the ratio across tests
        const averageRatio = totalRatio / testCount;

        console.log(`My algorithm was ${(((totalRatio/totalCRatio))*100).toFixed(3)}% better then Christophides`)
        console.log(`My algorithm was ${(((totalRatio/testCount)-1)*100).toFixed(3)}% worse then MST`)

    }
};