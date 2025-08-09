//This code is ugly and inefficient and I hate it

import { Hono } from "hono";
import { serveStatic } from 'hono/serve-static'
import { join } from 'https://deno.land/std@0.224.0/path/mod.ts'

const app = new Hono();
app.use(
  '/*',
  serveStatic({
    root: '../frontend', // relative to cwd
    getContent: async (path) => {
      const filePath = join('../frontend', path)
      try {
        return await Deno.readFile(filePath)
      } catch {
        return null // File not found
      }
    },
  })
)
class Point {
  x: number;
  y: number;
  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }
}

app.post("/solve", async (c) => {
  // Directly parse JSON body as an object
  const start = performance.now();
  const body = await c.req.json() as { pts: { x: number; y: number }[] };
  const points = parseToPoints(body);
  await writeFile(points);
  const res = await Deno.readTextFile("output/OUT.tsp");
  const end = performance.now() - start;
  const response = { pts: parseFileToPoints(res), time: end };
  console.log("Sending response to client ", response);
  return c.json(response);
});
async function sendHtml(c: any, filename: string) {
  const filePath = join('../frontend', filename)
  try {
    const html = await Deno.readTextFile(filePath)
    return c.html(html)
  } catch {
    return c.text('404 Not Found', 404)
  }
}

// Routes
app.get('/', (c) => sendHtml(c, 'index.html'))
app.get('/about', (c) => sendHtml(c, 'about.html'))
app.get('/data', (c) => sendHtml(c, 'data.html'))
function parseFileToPoints(input: string): Point[] {
  const section = input.split("NODE_COORD_SECTION")[1];
  if (!section) return [];

  const lines = section.trim().split("\n").filter(line => line.trim().length > 0);

  const points: Point[] = [];

  for (const line of lines) {
    const parts = line.trim().split(/\s+/);
    if (parts.length < 3) continue;  // skip malformed line

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
    // Typically TSP node indices start at 1
    str += `${i + 1}    ${input[i].x}    ${input[i].y}\n`;
  }

  // Await writing file as it returns a Promise
  await Deno.writeTextFile("input/IN.tsp", str);

  // Spawn the solver process, wait for it to finish if you want
  const command = new Deno.Command("../solver/target/release/tsp_rust.exe", {
    args: ["input/IN.tsp", "--no-log", "--no-post"],
  });

  const child = command.spawn();
  await child.status; // wait for the process to finish if needed
}

Deno.serve(app.fetch);
