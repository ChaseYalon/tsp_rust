import { brute } from "./brute.mjs";
import util from "./util.mjs";
import { christophides } from "./christophides.mjs";
import { nearest } from "./nearesteNeighbor.mjs";

//import { global } from "./globals.mjs";
//import { canvasEffects } from "./canvasEffects.mjs";

const resultDiv = document.getElementById("result");
const submit = document.getElementById("submit");


submit.addEventListener("click", async () => {
    resultDiv.append(await createTable());
});

async function createTable() {
    let tbl = document.createElement("table");
    let tblBody = document.createElement("tbody");
    document.getElementById("note").innerHTML=""


    let bruteDistance = 0;
    let ittDistance = 0;
    let chrisDistance = 0;
    let nnDist = 0;

    let bruteTime = 0;
    let ittTime = 0;
    let chrisTime = 0;
    let nnTime = 0;

    // Ensure numeric values from inputs
    const testCount = parseInt(document.getElementById("testCount").value) || 0;
    const pointCount = parseInt(document.getElementById("pointCount").value) || 0;

    for (let i = 0; i < testCount; i++) {
        let metaStart = performance.now()
        let localPoints = [];
        for (let j = 0; j < pointCount; j++) {
            localPoints.push({ x: util.rand(0, 400), y: util.rand(0, 400) });
        }
        let start;
        if(pointCount>10){
            start = performance.now();
            bruteDistance+=util.ePathDist(brute.mst(localPoints));
            bruteTime+=performance.now()-start
        }else{

            start = performance.now();
            bruteDistance += brute.tspBruteForce(localPoints).shortestDistance;
            bruteTime += performance.now() - start;
        }


        start = performance.now();
        chrisDistance += util.pathDist(christophides.christofidesTSP(localPoints));
        let chrisTA = performance.now()-start
        chrisTime += chrisTA; // Fixed typo from `chirsTime` to `chrisTime`

        let response = await fetch(`/solve`,{
            method: "POST",
            body: JSON.stringify({pts: localPoints})
        })
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        let data = await response.json(); // Parse JSON response
        let hull = data.pts; // Extract hull
        ittDistance += util.pathDist(hull);

        ittTime += data.time;

        start = performance.now();
        nnDist += util.pathDist(nearest.neighbor(localPoints));
        nnTime+=performance.now()-start;
        console.log("We are ",(100*(i/testCount)).toFixed(2),"% done, time taken ",(performance.now()-metaStart).toFixed(2)," ms itt time is ",data.time.toFixed(2), "while christ time is ",chrisTA.toFixed(2));

    }

    const header = document.createElement("tr");
    header.appendChild(createCell("Algorithm Name", true));
    header.appendChild(createCell("Percent of Brute Dist", true));
    header.appendChild(createCell("Total Execution Time", true));
    tblBody.appendChild(header);

    const bruteRow = document.createElement("tr");
    bruteRow.appendChild(createCell(`${pointCount>10 ? "Minnimum Spanning Tree" : "Brute Force"}`));
    bruteRow.appendChild(createCell("100%"));
    bruteRow.appendChild(createCell(`${(bruteTime / 1000).toFixed(2)} seconds`));
    tblBody.appendChild(bruteRow);

    const chrisRow = document.createElement("tr");
    chrisRow.appendChild(createCell("Christophides Algorithm"));
    chrisRow.appendChild(createCell(`${(100 * chrisDistance / bruteDistance).toFixed(2)}%`));
    chrisRow.appendChild(createCell(`${(chrisTime / 1000).toFixed(2)} seconds`));
    tblBody.appendChild(chrisRow);

    
    const nnrow = document.createElement("tr");
    nnrow.appendChild(createCell("Nearest Neighbor"));
    nnrow.appendChild(createCell(`${100*(nnDist/bruteDistance).toFixed(2)}%`));
    nnrow.appendChild(createCell(`${(nnTime / 1000).toFixed(2)} seconds`));
    tblBody.appendChild(nnrow);

    const ittRow = document.createElement("tr");
    ittRow.appendChild(createCell("My Algorithm"));
    ittRow.appendChild(createCell(`${(100 * ittDistance / bruteDistance).toFixed(2)}%`));
    ittRow.appendChild(createCell(`${(ittTime / 1000).toFixed(2)} seconds`));
    tblBody.appendChild(ittRow);


    tbl.appendChild(tblBody);

    // Helper function to create a table cell
    function createCell(content, isHeader = false) {
        const cell = document.createElement(isHeader ? "th" : "td");
        cell.textContent = content;
        return cell;
    }
    if(pointCount>10){
        document.getElementById("note").innerHTML="This is the minnimum spanning tree, it is less then the brute force by about 3%"
    }
    return tbl;
}
