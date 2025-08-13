import { BFManager, /*bruteForce*/ } from "./js/bruteforce.mjs";
import { Custom_Canvas } from "./js/canvas.mjs";

export class Point {
    x;
    y;
    constructor(x, y) {
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
const brutebtn = document.getElementById('brute');
const drawable = new Custom_Canvas(canvas);

// Better state management


const bruteForce = new BFManager(brutebtn);

drawable.onClick(() => {
    drawable.drawCircle(drawable.getMouseX(), drawable.getMouseY());
    points.push(new Point(drawable.getMouseX(), drawable.getMouseY()));
    bruteForce.checkPointLimit(points.length);
});

/**
 * @param {Point[]} points - set of points to solve, requires HTTP
 * @returns {Point[]} - Solved point set
 */
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
    const data = jsonData.pts;
    return data;
}

function rand(min, max) {
    return Math.random() * (max - min) + min;
}

solvebtn.addEventListener('click', async () => {
    const path = await solve(points);
    console.log(path);
    drawable.drawPath(path);
});

clearbtn.addEventListener('click', () => {
    drawable.onInit();
    points = [];

});

addbtn.addEventListener('click', () => {
    let pt = new Point(rand(0, 800), rand(0, 600));
    points.push(pt);
    drawable.drawCircle(pt.x, pt.y);
    bruteForce.checkPointLimit(points.length);
});

brutebtn.addEventListener('click', () => {
    console.log("Brute force enabled:", bruteForce.isEnabled);
    if (points.length <= 15) {
        bruteForce.toggle();
    }
    // If > 15 points, button stays disabled (no action)
});