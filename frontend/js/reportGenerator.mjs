import { christofidesAlgo } from "./util_algos.mjs";

async function solve(points) {
    let to_send = JSON.stringify({pts: points});
    const response = await fetch("/solve", {
        method: "POST",
        body: to_send
    });
    if (!response.ok) {
        throw new Error("HTTP request failed");
    }
    const jsonData = await response.json();
    return jsonData;
}
/**
 * 
 * @param {Edge} a - Edge with to and from points
 */
function edgeDist(a){
    return calcDist(a.from, a.to);
}
function edgePathDist(edges){
    let dist = 0;
    for(let i = 0; i < edges.length; i++){
        dist += edgeDist(edges[i]);
    }
    return dist;
}
/**
 * 
 * @param {Point[]} points - Path to find the distance of
 * @returns {number} - length of the tour
 * @description finds length of a tour
 */
function pathDist(points){
    let dist = 0;
    for(let i = 0; i < points.length - 1; i++){
        dist += calcDist(points[i], points[i + 1]);
    }
    dist += calcDist(points[points.length - 1], points[0]);
    return dist
}
/**
 * 
 * @param {Point} a - One point
 * @param {Point} b - Other Point
 * @returns {number} - euclidean distance between a and b
 * @description Returns distance between a and b
 */
function calcDist(a, b){
    return Math.sqrt(((a.x - b.x) * (a.x - b.x)) + ((a.y - b.y) * (a.y - b.y)));
}

class Algorithm {
    /**
     * 
     * @param {string} name - Name of the algorithm
     * @param {bool} isOptimal - Represents if the algorithm is optimal
     * @param {async (Point[][], number, HTMLProgressElement) => {number, number}} runner - Runner of the function, takes in a point count and a test count and returns a cumulativeDist and a cumulativeTime in ms
     */
    constructor(name, isOptimal, runner){
        this.name = name;
        this.isOptimal = isOptimal;
        this.runner = runner;
    }
    static redirect(){
        window.location.href = "/server-err";
    }
}

const concorde = new Algorithm('Concorde', true, async (gPoints, testCount, pb) => {
    let cumulativeDist = 0;
    let totalTime = 0;
    for(let i = 0; i < testCount; i++){
        const response = await fetch("/brute", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(gPoints[i])
        });


        const jsonData = await response.json();
        cumulativeDist += jsonData.dist;
        totalTime += jsonData.time;
        pb.value++;
    }
    //Adjust from s to ms
    totalTime *= 1000
    return {cumulativeDist, totalTime};
})

const mecum = new Algorithm('Mecum', false, async (gPoints, testCount, pb) => {
    let cumulativeDist = 0;
    let totalTime = 0;
    for(let i = 0; i < testCount; i++){
        let points = [...gPoints[i]];
        let res = await solve(points);

        totalTime += res.time;
        cumulativeDist += pathDist(res.pts);
        pb.value++;
    }
    return { cumulativeDist, totalTime };
})

const lkh = new Algorithm('Lin-Kernighan Heuristic', false, async (gPoints, testCount, pb) => {
    let cumulativeDist = 0;
    let totalTime = 0;
    for(let i = 0; i < testCount; i++){
        let points = [...gPoints[i]]
        const response = await fetch("/lkh", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(points)
        });


        const jsonData = await response.json();
        cumulativeDist += jsonData.dist;
        totalTime += jsonData.time;
        pb.value++;
    }
    //Adjust from s to ms
    totalTime *= 1000
    return {cumulativeDist, totalTime};
})

const christ = new Algorithm('Christofides Algorithm**', false, async (gPoints, testCount, pb) => {
    let cumulativeDist = 0;
    let start = performance.now();
    for(let i = 0; i < testCount; i++){
        let points = [...gPoints[i]];
        cumulativeDist += pathDist(christofidesAlgo(points));
        pb.value++;
    }
    let totalTime = performance.now() - start;
    return { cumulativeDist, totalTime };
})
export class ReportGenerator {
    /**
     * 
     * @param {HTMLButtonElement} trigger - Button to trigger report generator on click
     * @param {HTMLDivElement} output - Div to write output to
     * @param {HTMLInputElement} ldachek - checkbox to indicate if algorithm should be run
     * @param {HTMLInputElement} bfcheck - checkbox to indicate if algorithm should be run
     * @param {HTMLInputElement} lkhcheck- checkbox to indicate if algorithm should be run
     * @param {HTMLInputElement} cacheck - checkbox to indicate if algorithm should be run
     * @param {HTMLInputElement} pointCount - Represents how many points per test
     * @param {HTMLInputElement} testCount - Represents how many tests per point
     */
    constructor(output, ldachek, bfcheck, lkhcheck, cacheck, pointCount, testCount){
        this.output = output;
        this.ldachek = ldachek;
        this.bfcheck = bfcheck;
        this.lkhcheck = lkhcheck;
        this.cacheck = cacheck;
        this.pointCount = pointCount;
        this.testCount = testCount;

        this.algoArray = [this.ldachek.checked, this.bfcheck.checked, this.lkhcheck.checked, this.cacheck.checked];
        this.ldachek.addEventListener('click', () => {this.algoArray[0] = this.ldachek.checked});
        this.bfcheck.addEventListener('click', () => {this.algoArray[1] = this.bfcheck.checked});
        this.lkhcheck.addEventListener('click', () => {this.algoArray[2] = this.lkhcheck.checked});
        this.cacheck.addEventListener('click', () => {this.algoArray[3] = this.cacheck.checked});
    }
    async generate(){
        // <Initialize pont and test count>
        let pointCountStr = this.pointCount.value;
        let testCountStr = this.testCount.value;
        let pointCount;
        let testCount;
        try {
            pointCount = parseInt(pointCountStr);
            testCount = parseInt(testCountStr);
            if(pointCount == NaN || pointCount == NaN){
                throw new Error();
            }
        } catch {
            alert("Point and test count must be numbers, please try again");
            throw new Error("Point and test count not numbers");
        }
        pointCount = parseInt(pointCountStr);
        testCount = parseInt(testCountStr);
        if (isNaN(pointCount) || isNaN(testCount)) {
            alert("Point and test count must be numbers, please try again");
            throw new Error("Point and test count not numbers");
        }
        let globalPoints = new Array(testCount).fill(0).map(() => new Array(pointCount).fill(0).map(() => ({x: Math.random() * 800,y: Math.random() * 500})));
        // </Initialize point and test count>

        // <Create table>
        let table = document.createElement("table");
        let tbody = document.createElement("tbody");

        //Create label row
        let labelRow = document.createElement("tr");
        const labels = ["Algorithm Name", "Execution Time (S)", "Approx % above optimal*"];
        labels.forEach(label => {
            const cell = document.createElement("td");
            cell.textContent = label;
            labelRow.appendChild(cell);
        });
        tbody.appendChild(labelRow);
        // </Create table>

        //Docker sucks
        // Define algorithms with their configurations
        const algorithms = [concorde, mecum, lkh, christ];
        concorde.enabled = true;
        this.algoArray[0] ? mecum.enabled = true : mecum.enabled = false;
        this.algoArray[2] ? lkh.enabled = true : lkh.enabled = false;
        this.algoArray[3] ? christ.enabled = true: christ.enabled = false;
        const pb = document.createElement('progress');
        let algosEnabledCount = this.algoArray.reduce((acc, e) => acc + (e ? 1 : 0), 0) - 1;
        pb.max = algosEnabledCount * testCount;
        pb.value = 0;
        this.output.appendChild(pb);

        // Run enabled algorithms
        for (const algo of algorithms) {
            if (!algo.enabled) continue;
            // Check point limit for brute force
            if (algo.maxPoints && pointCount > algo.maxPoints) {
                alert(`Please disable ${algo.name} for tests with greater than ${algo.maxPoints} points`);
                throw new Error(`Point count too great for ${algo.name}`);
            }

            const result = await algo.runner(globalPoints, testCount, pb);
            
            // Store brute force distance for comparison
            if (algo.isOptimal) {
                this.bfDist = result.cumulativeDist;
            }

            // Create table row
            const row = document.createElement('tr');
            console.log("res", result);
            const rowData = [
                algo.name,
                //Finds average among tests then converts to seconds
                `${((result.totalTime / testCount) /1000).toFixed(2)}`,
                algo.isOptimal ? "100.00%" : `${((result.cumulativeDist / this.bfDist) * 100).toFixed(2)}%`
            ];

            rowData.forEach(data => {
                const cell = document.createElement('td');
                cell.textContent = data;
                row.appendChild(cell);
            });

            tbody.appendChild(row);
        }
        table.appendChild(tbody);
        this.output.innerHTML = "";
        this.output.appendChild(table);
    }
}

const reptcont = document.getElementById("report-container");
const ldachek = document.getElementById("lda-check");
const bfcheck = document.getElementById('bf-check');
const lkhcheck = document.getElementById('nn-check');
//Christofides algorithm check
const cacheck = document.getElementById('chris-check');
const pointCount = document.getElementById('pt-count');
const testCount = document.getElementById('test-count');

let generator = new ReportGenerator(reptcont, ldachek, bfcheck, lkhcheck, cacheck, pointCount, testCount);
window.addEventListener('workerReady', async () => {
    console.log('Worker is ready, now call generate()');
    await generator.generate();
    self.close();
});
