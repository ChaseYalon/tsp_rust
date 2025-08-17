const generatebtn = document.getElementById("generate");
const reptct = document.getElementById("report-container")
document.addEventListener('DOMContentLoaded', function() {
    const workerEvent = new Event('workerReady');

    generatebtn.addEventListener('click', () => {
        const worker = new Worker('./js/reportWorker.mjs', { type: 'module' });
        window.worker = worker;

        // Dispatch event
        window.dispatchEvent(workerEvent);
    });
})