import { canvasEffects } from "./canvasEffects.mjs";
import { brute } from "./brute.mjs";
import { global,Point } from "./globals.mjs";


const util = {
    disable(){
        if(global.bools.isBrute){
            global.html.isBruteButton.className="notBrute"
            global.html.isBruteButton.innerHTML = "Brute Force Disabled"
        }else if(!global.bools.isBrute&&global.arrs.points.length<10){
            global.html.isBruteButton.className="isBrute"
            global.html.isBruteButton.innerHTML = "Brute Force Enabled"

        }
        global.bools.isBrute = !global.bools.isBrute;

    },
    processPoints(points){
        let xS = [];
        let yS = [];
        for(let i=0;i<points.length;i++){
            xS.push(points[i].x);
            yS.push(poitns[i].y);
        }
        return {xS:xS,yS:yS};
    },
    ePathDist(edges){
        let sum=0;
        for(let i=0;i<edges.length;i++){
            sum+=edges[i][2]
        }
        return sum;
    },
    rand(min, max) {
        return parseFloat((Math.random() * (max - min) + min).toFixed(2));
    },
    addPoint(x,y) {
        
        canvasEffects.drawCircle(x, y);
        const toAdd = new Point(x, y);
        global.arrs.points.push(toAdd);
        global.arrs.unusedPoints.push(toAdd);
        if(global.arrs.points.length>10&&!global.bools.prevDis){
            util.disable();
            global.bools.prevDis=true
        }
    },
    calcDist(p1, p2) {
        return Math.sqrt(Math.pow(p2.y - p1.y, 2) + Math.pow(p2.x - p1.x, 2));
    },
    pathDist(path){
        let dist = 0;
        for(let i=0; i<path.length-1;i++){
            dist+=util.calcDist(path[i],path[i+1]);
        }
        dist+=util.calcDist(path[path.length-1],path[0])
        return dist;
    },
    pointSet(){
        if(global.bools.isBrute){
            let toSet={
                points:global.bools.points,
                ittDist:util.pathDist(hull),
                bruteDist:global.arrs.bruteRoute.shortestDistance,
                ittHull:global.arrs.hull,
                bruteHull:global.arrs.bruteRoute.bestRoute

            }
            document.getElementById("toCopy").innerHTML=JSON.stringify(toSet);
        }else{
            throw new Error("Brue force is not enabled")
        }

    },

    async solve() {
        let startTime = performance.now();
    
        // Fetch the solution from the backend
        let response = await fetch(`/solve`,{
            method: "POST",
            body: JSON.stringify({pts: global.arrs.points})
        })

        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        
        let data = await response.json(); // Parse JSON response
        let hull = data.pts; // Extract hull
        let time = data.time; // Extract time
        

        
        //hull = JSON.parse(hull);
    
        canvasEffects.drawPath(hull, "red", global.html.ctx);
    
        if (global.bools.isBrute) {
            if (global.arrs.points.length > 10) {
                throw new Error(`It should not be executing. There are ${global.arrs.points.length} points. isBrute is ${global.bools.isBrute}`);
            }
    
            let startBrute = performance.now();
            global.arrs.bruteRoute = brute.tspBruteForce(global.arrs.points);
            let endBrute = performance.now();
            console.log("Brute rotue is ",global.arrs.bruteRoute);
            canvasEffects.drawPath(global.arrs.bruteRoute.bestRoute, "blue", global.html.ctx2);
    
            let bruteTime = (endBrute - startBrute).toFixed(3);
            global.html.brutePath.innerHTML = `${global.arrs.bruteRoute.shortestDistance.toFixed(3)} the time is ${bruteTime}`;
    
            if (global.arrs.bruteRoute.shortestDistance - util.pathDist(hull) > 0.0001 &&
                global.arrs.bruteRoute.shortestDistance > util.pathDist(hull)) {
                util.pointSet();
                alert(`Discrepancy detected: ${global.arrs.bruteRoute.shortestDistance} (brute) vs. ${util.pathDist(hull)} (iterative)`);
                throw new Error("Discrepancy detected");
            }
        } else {
            let edges = brute.mst(global.arrs.points);
            canvasEffects.drawEdgePath(edges);
            global.html.brutePath.innerHTML = this.ePathDist(edges);
        }
    
        global.html.ittPath.innerHTML = `${util.pathDist(hull).toFixed(3)} the time is ${JSON.stringify(time.toFixed(3))}`;
    }
    

}

export default util;
