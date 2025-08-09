import { canvasEffects } from "./canvasEffects.mjs";
import  util  from "./util.mjs";
import {global} from "./globals.mjs"

export const onPress = {
    solveNTimes() {
        let n = parseInt(prompt("How many cycles would you like?"), 10);
        if (isNaN(n) || n <= 0) {
            alert("Please enter a valid positive number.");
            return;
        }

        for (let i = 0; i < n; i++) {
            canvasEffects.innit(); // Clear the canvas and reset points for each cycle

            // Add 9 points
            for (let j = 0; j < 9; j++) {
                util.addPoint(util.rand(0,500),util.rand(0,500));
            }

            util.solve(); // Solve the TSP problem
        }

        global.misc.keysPressed.clear();
    },
    addNPoints(){
        let n = prompt("How many points would you like to add");
        for(let i=0;i<n;i++){
            util.addPoint(util.rand(0,500),util.rand(0,500));
        }
        global.misc.keysPressed=new Set();
    }
}