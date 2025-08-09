//<imports>
import { canvasEffects } from "../jsFiles/canvasEffects.mjs";
import  util  from "../jsFiles/util.mjs";
import { onPress } from "../jsFiles/onPress.mjs";
import { global, Point } from "../jsFiles/globals.mjs";
//You can acess tests from the console
//</imports>


//<Event Listners>
document.addEventListener("keydown", (e) => {
    global.misc.keysPressed.add(e.code);

    if (global.misc.keysPressed.has("ControlLeft") || global.misc.keysPressed.has("ControlRight")) {
        if (global.misc.keysPressed.has("F6")) onPress.addNPoints();
        if (global.misc.keysPressed.has("F7")) onPress.solveNTimes();
        if (global.misc.keysPressed.has("F8")) {
            let input = prompt("This bulk test will use the mst.\nPlease enter how many points you would like, a comma and how many tests");
            let points = input.split(",")[0];
            let tests = input.split(",")[1];
            if (points != null && tests != null) {
                tests.test(true, points, tests);
            }
            global.misc.keysPressed.clear();
        }
    }
});

document.addEventListener("keyup", (e) => global.misc.keysPressed.delete(e.code));

document.getElementById('add').onclick = function() {
    const x = util.rand(0,500);
    const y = util.rand(0,500);
    util.addPoint(x,y)
}
document.getElementById('solveButton').onclick = function() {
    util.solve()
}
document.getElementById('clear').onclick = function() {
    canvasEffects.innit();
}
document.getElementById('pointSet').onclick = function() {
    util.pointSet()
}
document.getElementById('brute').onclick = function() {
    util.disable()
}

//</Event Listners>



canvasEffects.innit();

