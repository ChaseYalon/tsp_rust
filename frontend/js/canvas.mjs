class Point{
    constructor(x, y){
        this.x = x;
        this.y = y;
    }
}
export class Custom_Canvas {
    callback = () => {};
    /** 
    * @param {HTMLCanvasElement} canvas -  is the canvas that you want to use
    */
    constructor(canvas){
        this.canvas = canvas;
        this.height = this.canvas.height;
        this.width = this.canvas.width;
        this.ctx = this.canvas.getContext("2d");
        this.mouse_x = -1;
        this.mouse_y = -1;
        this.onInit();

        this.canvas.addEventListener('mousemove', (event) => {
            this.mouse_x = event.offsetX;
            this.mouse_y = event.offsetY;
        });
        this.canvas.addEventListener('mouseleave', () => {
            this.mouse_x = -1;
            this.mouse_y = -1;
        });
        this.canvas.addEventListener('click', () => {
            this.callback()
        });
    }
    /**
     * 
     * @param {number} x - x val of circle center
     * @param {number} y - y val of circle center
     * @param {number} radius - radius of circle, defaults to 7
     * @description draws circle to the canvas
     */
    drawCircle(x, y, radius = 7){
        this.ctx.beginPath();
        this.ctx.arc(x, y, radius, 0, 2 * Math.PI);
        this.ctx.strokeStyle = 'black';
        this.ctx.lineWidth = 2; 
        this.ctx.stroke(); 

        this.ctx.fillStyle = 'red';
        this.ctx.fill(); 
    }
    /**
     * 
     * @param {number} x1 -  First x coordinate
     * @param {number} y1 -  First y coordinate
     * @param {number} x2 -  Second x coordinate
     * @param {number} y2 -  Second y coordinate
     * @param {string} color - Either a name or an RGB / hex color value
     * @description draws rectangle to the screen
     */
    drawRect(x1, y1, x2, y2, color){
        this.ctx.fillStyle = color;
        let width = x2 - x1;
        let height = y2 - y1
        this.ctx.fillRect(x1, y1, width, height);
    }
    /**
     * @description run whenever you want to clear the canvas and reset its state
     */
    onInit(){
        console.log("Resting canvas");
        this.drawRect(0, 0, 800, 600, "white");
    }
    /**
     * 
     * @returns {boolean}
     * @description Returns true if mouse is over the canvas, otherwise false
     */
    mouseOnCanvas(){
        if (this.mouse_x == -1 && this.mouse_y == -1){
            return false;
        }
        return true
    }
    /**
     * 
     * @returns {number | null}
     * @description returns the mouse_x if it is over the canvas, otherwise null
     */
    getMouseX(){
        if(this.mouse_x == -1){
            return null;
        }
        return this.mouse_x
    }
        /**
     * 
     * @returns {number | null}
     * @description returns the mouse y if it is over the canvas, otherwise null
     */
    getMouseY(){
        if(this.mouse_y == -1){
            return null;
        }
        return this.mouse_y
    }
    /**
     * 
     * @param {(() => void)} callback - callback to run
     * @description Takes a callback that runs when the mouse is clicked on the canvas
     */
    onClick(callback = () =>{}){
        this.callback = callback;
    }
    /**
     * 
     * @param {number} x1 - first x coordinate
     * @param {number} y1 - first y coordinate
     * @param {number} x2 - second x coordinate
     * @param {number} y2 - second y coordinate
     * @param {*} color - color as a name or rgb/hex
     */
    drawLine(x1, y1, x2, y2, color){
        this.ctx.beginPath();
        this.ctx.moveTo(x1, y1); 
        this.ctx.lineTo(x2, y2); 
        this.ctx.lineWidth = 4;
        this.ctx.strokeStyle = color;
        this.ctx.stroke();
    }
    /**
     * 
     * @param {Point[]} path - path to draw
     * @param {string} color - color to draw the path (defaults to "black")
     * @returns {void}
     */
    drawPath(path, color = "black"){
        for(let i = 0; i < path.length - 1; i++){
            this.drawLine(path[i].x, path[i].y, path[i + 1].x, path[i + 1].y, color);
        }
        this.drawLine(path[path.length - 1].x, path[path.length - 1].y, path[0].x, path[0].y, color);
    }
}