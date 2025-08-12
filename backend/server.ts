import { Hono } from "hono";
import { serveStatic } from "hono/deno";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";

const app = new Hono();
const frontendRoot = join(Deno.cwd(), "frontend");

async function sendHtml(c: any, filename: string) {
  const filePath = join(frontendRoot, filename);
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
  await writeFile(points);
  const res = await Deno.readTextFile("output/OUT.tsp");
  const end = performance.now() - start;
  const response = { pts: parseFileToPoints(res), time: end };
  
  // Clean up input file
  try {
    await Deno.writeTextFile("input/IN.tsp", "");
  } catch {
    // Ignore if cleanup fails
  }
  
  console.log("Sending response to client");
  return c.json(response);
});

// Explicit HTML routes first
app.get("/", (c) => sendHtml(c, "index.html"));
app.get("/about", (c) => sendHtml(c, "about.html"));
app.get("/data", (c) => sendHtml(c, "data.html"));
// Serve static files (put this after explicit routes to avoid conflicts)
app.use("/static/*", serveStatic({ 
  root: "./" 
}));

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
  const toRet: Point[] = [];
  for (let i = 0; i < input.pts.length; i++) {
    toRet.push(new Point(input.pts[i].x, input.pts[i].y));
  }
  return toRet;
}

async function writeFile(input: Point[]): Promise<void> {
  let str = `NAME : to_solve
COMMENT : to be solved with tsp_solver (Copyright Chase Yalon)
TYPE : TSP
DIMENSION: ${input.length}
EDGE_WEIGHT_TYPE : EUC_2D
NODE_COORD_SECTION
`;

  for (let i = 0; i < input.length; i++) {
    str += `${i + 1}    ${input[i].x}    ${input[i].y}\n`;
  }

  // Ensure input and output directories exist
  try {
    await Deno.mkdir("input", { recursive: true });
  } catch {
    // Directory might already exist
  }
  
  try {
    await Deno.mkdir("output", { recursive: true });
  } catch {
    // Directory might already exist
  }

  await Deno.writeTextFile("input/IN.tsp", str);

  // Cross-platform executable path detection
  const isWindows = Deno.build.os === "windows";
  const executableName = isWindows ? "tsp_rust.exe" : "tsp_rust";
  
  // Try different possible paths based on the environment
  const possiblePaths = [
    `./solver/target/release/${executableName}`,  // Docker/container path
    `../solver/target/release/${executableName}`, // Local development path
    `solver/target/release/${executableName}`,    // Alternative relative path
  ];
  
  let executablePath = possiblePaths[0]; // Default to first option
  
  // Find the correct path by checking which file exists
  for (const path of possiblePaths) {
    try {
      const stat = await Deno.stat(path);
      if (stat.isFile) {
        executablePath = path;
        break;
      }
    } catch {
      // File doesn't exist, continue to next path
      continue;
    }
  }

  console.log(`Using executable path: ${executablePath}`);

  const command = new Deno.Command(executablePath, {
    args: ["input/IN.tsp"],
  });

  const child = command.spawn();
  await child.status;
}

const options = {
  port: 443,
  cert: await Deno.readTextFile("certs/fullchain.pem"),
  key: await Deno.readTextFile("certs/privkey.pem"),
};

Deno.serve(options, app.fetch);
