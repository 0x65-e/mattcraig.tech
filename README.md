# About

This website is written for Cloudflare Workers using Rust and the (workers-rs)[https://github.com/cloudflare/workers-rs] crate.

It serves static files stored in Workers KV.

## Usage

With `wrangler` CLI, you can build, test, and deploy to Workers with the following commands: 

```bash
# compiles project to WebAssembly and will warn of any issues
wrangler build 

# runs Worker in an ideal development workflow (with a local server, file watcher & more)
wrangler dev

# deploys Worker globally to Cloudflare
wrangler publish
```
