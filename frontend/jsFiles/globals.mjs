let ctx,ctx2,ittPath,brutePath,isBruteButton,addPointButton,pointSetButton,graph;


if (!window.location.href.includes("data")){
    //<HTML components>
     ctx = document.getElementById("canvas").getContext("2d");
     ctx2 = document.getElementById("canvas2").getContext("2d");
     ittPath = document.getElementById("path1");
     brutePath = document.getElementById("path2");
     isBruteButton = document.getElementById("brute");
     addPointButton = document.getElementById('add');
     pointSetButton = document.getElementById("pointSet");
     graph = document.getElementById("graph");
    ///</HTML components>
}


//<Arrs>
let points = [];
let unusedPoints = [];
let hull = [];
let bruteRoute = [];
//</Arrs>

//<Bools>
let isBrute = true
let prevDis=false
//</Bools>

//<Misc>
let keysPressed = new Set();
//</Misc>

export const global={
    html:{
        ctx:ctx,
        ctx2:ctx2,
        ittPath:ittPath,
        brutePath:brutePath,
        isBruteButton:isBruteButton,
        addPointButton:addPointButton,
        pointSetButton:pointSetButton,
        graph:graph
    },
    bools:{
        isBrute:isBrute,
        prevDis:prevDis
    },
    arrs:{
        points:points,
        unusedPoints:unusedPoints,
        hull:hull,
        bruteRoute:bruteRoute
    },
    misc:{
        keysPressed:keysPressed
    }
}
export class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
}