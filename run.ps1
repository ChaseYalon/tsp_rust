cd solver
cargo build --release
cd ../backend
deno run -A server.ts
cd ..
