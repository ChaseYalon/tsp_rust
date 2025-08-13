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

function init() {
  Deno.mkdirSync(INPUT_DIR, { recursive: true });
  Deno.mkdirSync(OUTPUT_DIR, { recursive: true });
  Deno.writeTextFileSync(join(INPUT_DIR, "IN.tsp"), "");
  Deno.writeTextFileSync(join(OUTPUT_DIR, "OUT.tsp"), "");
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

app.post("/solve", async (c) => {
  const start = performance.now();
  const body = (await c.req.json()) as { pts: { x: number; y: number }[] };
  const points = parseToPoints(body);


  await writeFileAndRunSolver(points);

  // Read solver output
  const outPath = join(OUTPUT_DIR, "OUT.tsp");
  const res = await Deno.readTextFile(outPath);

  const end = performance.now() - start;
  const response = { pts: parseFileToPoints(res), time: end };

  // Now it's safe to clean up
  await Deno.writeTextFile(join(INPUT_DIR, "IN.tsp"), "");

  return c.json(response);
});

// Explicit HTML routes first
app.get("/", (c) => sendHtml(c, "index.html"));
app.get("/about", (c) => sendHtml(c, "about.html"));
app.get("/data", (c) => sendHtml(c, "data.html"));

// Serve static files
app.use("/static/*", serveStatic({ root: "./" }));

// Catch-all for other static files
app.use("*", serveStatic({
  root: "./",
  rewriteRequestPath: (path) => path.replace(/^\//, "/frontend/")
}));

app.get("*", (c) => sendHtml(c, "errs/404.html"));

function parseFileToPoints(input: string): Point[] {
  const section = input.split("NODE_COORD_SECTION")[1];
  if (!section) return [];

  const lines = section.trim().split("\n").filter((line) => line.trim().length > 0);
  const points: Point[] = [];

  for (const line of lines) {
    const parts = line.trim().split(/\s+/);
    if (parts.length < 3) continue;

    const x = parseFloat(parts[1]);
    const y = parseFloat(parts[2]);

    if (!isNaN(x) && !isNaN(y)) {
      points.push(new Point(x, y));
    }
  }
  return points;
}

function parseToPoints(input: { pts: { x: number; y: number }[] }): Point[] {
  return input.pts.map(p => new Point(p.x, p.y));
}

async function writeFileAndRunSolver(input: Point[]): Promise<void> {
  const str =
`NAME : to_solve
COMMENT : to be solved with tsp_solver
TYPE : TSP
DIMENSION: ${input.length}
EDGE_WEIGHT_TYPE : EUC_2D
NODE_COORD_SECTION
${input.map((p, i) => `${i + 1}    ${p.x}    ${p.y}`).join("\n")}
`;

  const inPath = join(INPUT_DIR, "IN.tsp");
  await Deno.mkdir(INPUT_DIR, { recursive: true });
  await Deno.mkdir(OUTPUT_DIR, { recursive: true });

  await Deno.writeTextFile(inPath, str);

  const isWindows = Deno.build.os === "windows";
  const executableName = isWindows ? "tsp_rust.exe" : "tsp_rust";

  const possiblePaths = [
    join(APP_ROOT, "solver", "target", "release", executableName),
    join(APP_ROOT, "..", "solver", "target", "release", executableName),
    join("solver", "target", "release", executableName)
  ];

  let executablePath = possiblePaths[0];
  for (const path of possiblePaths) {
    try {
      const stat = await Deno.stat(path);
      if (stat.isFile) {
        executablePath = path;
        break;
      }
    } catch {}
  }


  const command = new Deno.Command(executablePath, {
    args: [inPath, "--no-log"],
    cwd: APP_ROOT, // force consistent working directory
  });

  const child = command.spawn();
  const status = await child.status;

  // Debug: list any OUT.tsp found
  const findCmd = new Deno.Command("find", {
    args: [APP_ROOT, "-name", "OUT.tsp"]
  });
  const findOut = await findCmd.output();
}

const options = {
  port: 443,
  cert: await Deno.readTextFile(join(APP_ROOT, "certs", "fullchain.pem")),
  key: await Deno.readTextFile(join(APP_ROOT, "certs", "privkey.pem")),
};

Deno.serve(options, app.fetch);
