import { Hono } from "hono";
import { serveStatic } from "hono/deno";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";

const app = new Hono();

// Absolute base directories
const APP_ROOT = Deno.cwd();
const FRONTEND_DIR = join(APP_ROOT, "frontend");
const BACKEND_DIR = join(APP_ROOT, "backend");
const INPUT_DIR = join(BACKEND_DIR, "input");
const OUTPUT_DIR = join(BACKEND_DIR, "output");

function ensureFile(path: string) {
  try {
    const stat = Deno.statSync(path);
    if (!stat.isFile) throw new Error();
  } catch {
    Deno.mkdirSync(join(path, ".."), { recursive: true });
    Deno.writeTextFileSync(path, "");
  }
}

function init() {
  Deno.mkdirSync(INPUT_DIR, { recursive: true });
  Deno.mkdirSync(OUTPUT_DIR, { recursive: true });
  ensureFile(join(INPUT_DIR, "IN.tsp"));
  ensureFile(join(OUTPUT_DIR, "OUT.tsp"));
}

init();

async function sendHtml(c: any, filename: string) {
  const filePath = join(FRONTEND_DIR, filename);
  try {
    const html = await Deno.readTextFile(filePath);
    return c.html(html);
  } catch {
    return c.text("404 Not Found", 404);
  }
}

class Point {
  x: number;
  y: number;
  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }
}

// --- Endpoints ---

app.post("/solve", async (c) => {
  const body = (await c.req.json()) as { pts: { x: number; y: number }[] };
  const points = parseToPoints(body);
  const start = performance.now();
  await writeFileAndRunSolver(points);
  const end = performance.now() - start;

  const outPath = join(OUTPUT_DIR, "OUT.tsp");
  const res = await Deno.readTextFile(outPath);

  return c.json({ pts: parseFileToPoints(res), time: end });
});

app.post("/brute", async (c) => {
  const points = (await c.req.json()) as Point[];
  if (!points || !Array.isArray(points) || points.length === 0) {
    return c.json({ error: "No points provided" }, 400);
  }

  await writeFile(points, join(INPUT_DIR, "BIN.tsp"));
  const command = new Deno.Command("/app/concorde/concorde", {
    args: ["backend/input/BIN.tsp"],
    cwd: APP_ROOT,
    stdout: "piped",
    stderr: "piped",
  });

  const output = await command.output();
  const stdoutText = new TextDecoder().decode(output.stdout);

  const solution = parseFloat(stdoutText.split("Optimal Solution: ")[1].split("\n")[0]);
  const runTime = parseFloat(stdoutText.split("Total Running Time: ")[1].split(" ")[0]);
  return c.json({ time: runTime, dist: solution });
});

app.post("/lkh", async (c) => {
  const points = (await c.req.json()) as Point[];
  if (!points || !Array.isArray(points) || points.length === 0) {
    return c.json({ error: "No points provided" }, 400);
  }

  const tspPath = join(INPUT_DIR, "LIN.tsp");
  await writeFile(points, tspPath);

  // Dynamically generate .par file with absolute paths
  const parPath = join(INPUT_DIR, "LIN.par");
  const parContent = `
PROBLEM_FILE = ${tspPath}
MOVE_TYPE = 5
PATCHING_C = 3
RUNS = ${points.length / 100}
`;
  await Deno.writeTextFile(parPath, parContent);

  const command = new Deno.Command("./app/lkh/lkh", {
    args: [parPath],
    cwd: APP_ROOT,
    stdout: "piped",
    stderr: "piped",
  });

  const output = await command.output();
  const stdoutText = new TextDecoder().decode(output.stdout);
  const stderrText = new TextDecoder().decode(output.stderr);

  if (stderrText) console.error(stderrText);

  const distanceMatch = stdoutText.match(/Cost\.min = ([0-9.]+)/);
  const timeMatch = stdoutText.match(/Time\.total = ([0-9.]+)/);

  if (!distanceMatch || !timeMatch) {
    return c.json({ error: "Failed to parse LKH output" }, 500);
  }

  const distance = parseFloat(distanceMatch[1]);
  const time = parseFloat(timeMatch[1]);

  return c.json({ time, dist: distance });
});

// --- HTML routes ---
app.get("/", (c) => sendHtml(c, "index.html"));
app.get("/about", (c) => sendHtml(c, "about.html"));
app.get("/data", (c) => sendHtml(c, "data.html"));
app.get("/server-err", (c) => sendHtml(c, "errs/500.html"));
// --- Static ---
app.use("/static/*", serveStatic({ root: "./" }));
app.use("*", serveStatic({ root: "./", rewriteRequestPath: (path) => path.replace(/^\//, "/frontend/") }));
app.get("*", (c) => sendHtml(c, "errs/404.html"));

// --- Helpers ---
function parseFileToPoints(input: string): Point[] {
  const section = input.split("NODE_COORD_SECTION")[1];
  if (!section) return [];
  return section
    .trim()
    .split("\n")
    .filter((l) => l.trim().length > 0)
    .map((line) => {
      const parts = line.trim().split(/\s+/);
      return new Point(parseFloat(parts[1]), parseFloat(parts[2]));
    })
    .filter((p) => !isNaN(p.x) && !isNaN(p.y));
}

function parseToPoints(input: { pts: { x: number; y: number }[] }): Point[] {
  return input.pts.map((p) => new Point(p.x, p.y));
}

async function writeFile(input: Point[], path: string) {
  const str = `NAME : to_solve
COMMENT : to be solved with tsp_solver
TYPE : TSP
DIMENSION: ${input.length}
EDGE_WEIGHT_TYPE : EUC_2D
NODE_COORD_SECTION
${input.map((p, i) => `${i + 1}    ${p.x}    ${p.y}`).join("\n")}
EOF`;

  await Deno.writeTextFile(path, str);
}

async function writeFileAndRunSolver(input: Point[]): Promise<void> {
  const inPath = join(INPUT_DIR, "IN.tsp");
  await Deno.mkdir(INPUT_DIR, { recursive: true });
  await Deno.mkdir(OUTPUT_DIR, { recursive: true });

  await writeFile(input, inPath);

  const isWindows = Deno.build.os === "windows";
  const executableName = isWindows ? "tsp_rust.exe" : "tsp_rust";

  const possiblePaths = [
    join(APP_ROOT, "solver", "target", "release", executableName),
    join(APP_ROOT, "..", "solver", "target", "release", executableName),
    join("solver", "target", "release", executableName),
  ];

  let executablePath = possiblePaths[0];
  for (const path of possiblePaths) {
    const stat = await Deno.stat(path);
    if (stat.isFile) {
      executablePath = path;
      break;
    }
    
  }

  const command = new Deno.Command(executablePath, {
    args: [inPath],
    cwd: APP_ROOT,
  });

  const { code, stderr } = await command.output();
  if (code !== 0) {
    console.error("Solver error:", new TextDecoder().decode(stderr));
    throw new Error("Solver failed");
  }
}

// --- Serve ---
const options = {
  port: 443,
  cert: await Deno.readTextFile(join(APP_ROOT, "certs", "fullchain.pem")),
  key: await Deno.readTextFile(join(APP_ROOT, "certs", "privkey.pem")),
};

Deno.serve(options, app.fetch);
