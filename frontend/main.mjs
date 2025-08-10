import { Custom_Canvas } from "./js/canvas.mjs";

export class Point{
    x;
    y;
    constructor(x, y){
        this.x = x;
        this.y = y;
    }
}
let points = [];
/* set up drawing */
const canvas = document.getElementById("main-canvas");
const solvebtn = document.getElementById('solve');
const clearbtn = document.getElementById('clear');
const addbtn = document.getElementById('add');
const drawable = new Custom_Canvas(canvas);

drawable.onClick(()=>{
    drawable.drawCircle(drawable.getMouseX(), drawable.getMouseY());
    points.push(new Point(drawable.getMouseX(), drawable.getMouseY()));
})

/**
 * 
 * @param {Point[]} points - set of points to solve, requires HTTP
 * @returns {Point[]} - Solved point set
 */

async function solve(points){
    let to_send = JSON.stringify({pts: points});
    const response = await fetch("/solve", {
        method: "POST",
        body: to_send
    })
    if (!response.ok){
        throw new Error("HTTP request failed")
    }
    const jsonData = await response.json();
    const data = jsonData.pts;
    return data;

}
function rand(min, max) {
  return Math.random() * (max - min) + min;
}

solvebtn.addEventListener('click', async () => {
    drawable.drawPath(await solve(points));
})

clearbtn.addEventListener('click', () => {
    drawable.onInit();
    points = [];
})

addbtn.addEventListener('click', () => {
    let pt = new Point(rand(0, 800), rand(0, 600));
    points.push(pt);
    drawable.drawCircle(pt.x, pt.y);
})