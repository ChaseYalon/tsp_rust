export class BFManager {
    constructor(button) {
        this.button = button;
        this.enabled = true;
    }

    get isEnabled() {
        return this.enabled;
    }

    enable() {
        this.enabled = true;
        this.button.classList.remove('bf-off');
        this.button.classList.add('bf-but');
        this.button.innerHTML = "Brute Force Enabled";
        this.checkPointLimit();
    }

    disable() {
        this.enabled = false;
        this.button.classList.remove('bf-but');
        this.button.classList.add('bf-off');
        this.button.innerHTML = "Brute Force Disabled";
    }

    toggle() {
        if (this.enabled) {
            this.disable();
        } else {
            this.enable();
        }
    }

    // Disable if too many points (>15)
    checkPointLimit(pointCount) {
        if (pointCount > 15 && this.enabled) {
            this.disable();
        }
    }
}
