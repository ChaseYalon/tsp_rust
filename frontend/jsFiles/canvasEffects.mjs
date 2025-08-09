import { global } from "./globals.mjs";



export const canvasEffects = {
    drawCircle(x, y) {
        global.html.ctx.beginPath();
        global.html.ctx.arc(x, y, 5, 0, 2 * Math.PI);
        global.html.ctx.fillStyle = "red";
        global.html.ctx.fill();
        global.html.ctx.strokeStyle = "red";
        global.html.ctx.lineWidth = 0;
        global.html.ctx.stroke();

        global.html.ctx2.beginPath();
        global.html.ctx2.arc(x, y, 5, 0, 2 * Math.PI);
        global.html.ctx2.fillStyle = "blue";
        global.html.ctx2.fill();
        global.html.ctx2.strokeStyle = "blue";
        global.html.ctx2.lineWidth = 0;
        global.html.ctx2.stroke();
    },
    drawEdgePath(edges){
        for(let i=0;i<edges.length;i++){
            let currentEdge = edges[i]
            canvasEffects.drawLine(global.arrs.points[currentEdge[0]].x,global.arrs.points[currentEdge[0]].y,global.arrs.points[currentEdge[1]].x,global.arrs.points[currentEdge[1]].y,"blue",global.html.ctx2)
        }
    },
    drawLine(x1, y1, x2, y2, color,ctx) {
 
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.strokeStyle = color;
        ctx.lineWidth = 3;
        ctx.stroke();
    },

    innit() {
        console.log("calling innit")
        global.bools.prevDis=false;
        global.arrs.points = [];
        global.arrs.unusedPoints = [];
        global.html.ittPath.innerHTML = "Itterative path will appear here";
        if(global.bools.isBrute){
            global.html.brutePath.innerHTML = "Brute force path will appear here"
        }
        global.html.ctx.beginPath();
        global.html.ctx.lineWidth = "0";
        global.html.ctx.strokeStyle = "red";
        global.html.ctx.fillStyle = "white";
        global.html.ctx.fill();
        global.html.ctx.fillRect(0, 0, 500, 500);
        global.html.ctx.stroke();

        global.html.ctx2.beginPath();
        global.html.ctx2.lineWidth = "0";
        global.html.ctx2.strokeStyle = "red";
        global.html.ctx2.fillStyle = "white";
        global.html.ctx2.fill();
        global.html.ctx2.fillRect(0, 0, 500, 500);
        global.html.ctx2.stroke();
        document.getElementById("toCopy").innerHTML="";
    },
    drawPath(path, color,ctx) {
        
        for (let i = 0; i < path.length - 1; i++) {
            canvasEffects.drawLine(path[i].x, path[i].y, path[i + 1].x, path[i + 1].y, color,ctx);
        }
        canvasEffects.drawLine(path[path.length - 1].x, path[path.length - 1].y, path[0].x, path[0].y, color,ctx);
    },

}