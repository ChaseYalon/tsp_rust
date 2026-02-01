import { BFManager } from "./js/bruteforce.mjs";
import { Custom_Canvas } from "./js/canvas.mjs";
import { solveBruteForce } from "./js/util_algos.mjs";

export class Point {
    x;
    y;
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
}

let points = [];
let ldaPath = null;
let bruteForcePath = null;
let showingBruteForce = false;

/* set up drawing */
const canvas = document.getElementById("main-canvas");
const solvebtn = document.getElementById('solve');
const clearbtn = document.getElementById('clear');
const addbtn = document.getElementById('add');
const brutebtn = document.getElementById('brute');
const togglePathBtn = document.getElementById('toggle-path');
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
    // Get LDA path from server
    ldaPath = await solve(points);
    console.log("LDA Path:", ldaPath);
    
    // Calculate brute force path if enabled
    if (bruteForce.isEnabled && points.length <= 15) {
        console.log("Computing brute force path...");
        bruteForcePath = solveBruteForce(points);
        console.log("Brute Force Path:", bruteForcePath);
        
        // Enable toggle button
        togglePathBtn.disabled = false;
        togglePathBtn.classList.remove('opt-but-disabled');
        togglePathBtn.classList.add('opt-but');
    } else {
        bruteForcePath = null;
        togglePathBtn.disabled = true;
        togglePathBtn.classList.remove('opt-but');
        togglePathBtn.classList.add('opt-but-disabled');
    }
    
    // Redraw based on current view
    redrawPaths();
});

clearbtn.addEventListener('click', () => {
    drawable.onInit();
    points = [];
    ldaPath = null;
    bruteForcePath = null;
    showingBruteForce = false;
    togglePathBtn.disabled = true;
    togglePathBtn.classList.remove('opt-but');
    togglePathBtn.classList.add('opt-but-disabled');
    updateToggleButton();
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

togglePathBtn.addEventListener('click', () => {
    showingBruteForce = !showingBruteForce;
    updateToggleButton();
    redrawPaths();
});

function updateToggleButton() {
    if (showingBruteForce) {
        togglePathBtn.innerHTML = "Showing: Brute Force";
    } else {
        togglePathBtn.innerHTML = "Showing: LDA";
    }
}

function redrawPaths() {
    // Clear canvas and redraw points
    drawable.onInit();
    points.forEach(pt => drawable.drawCircle(pt.x, pt.y));
    
    // Draw the appropriate path
    if (showingBruteForce && bruteForcePath) {
        drawable.drawPath(bruteForcePath, "blue");
    } else if (ldaPath) {
        drawable.drawPath(ldaPath, "black");
    }
}