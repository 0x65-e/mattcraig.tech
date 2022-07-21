# About

This website is written for Cloudflare Workers using Rust and the [workers-rs](https://github.com/cloudflare/workers-rs) crate. Cloudflare workers is a serverless, "function as a service" (FaaS) platform that runs across distributed data centers.

It serves static files stored in Workers KV, a serverless key-value store on the edge.

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

You will need to generate your own KV namespace and replace the values in [wrangler.toml](wrangler.toml).

```bash
# creates a preview namespace
wrangler kv:namespace create "STATIC" --preview

# creates a production namespace
wrangler kv:namespace create "STATIC"
```

You can choose a name other than `STATIC` for your namespace, but be sure to update the KV access in [libs.rs](src/lib.rs).

You may also want to change the name of your worker in [wrangler.toml](wrangler.toml).

## Static Files

Static files are stored in the `./static/` directory. To serve static assets, they must first be uploaded to Workers KV. This is possible to do in bulk using the included Powershell script [kv-bulk-upload.ps1](kv-bulk-upload.ps1). This keys each file with its relative path from the static directory (or whichever base directory you specify), which makes it easy to see your website layout on the filesystem. For example, the file `static/index.html` is mapped to the key `index.html` in the KV, while `static/assets/css/resume.css` becomes `assets/css/resume.css`.

You will need to enable Powershell scripts in the Windows security settings to run the upload script.

```powershell
# uploads all contents of ./static/ directory to preview namespace
./kv-bulk-upload.ps1 -preview

# uploads all contents of ./static/ directory to production namespace
./kv-bulk-upload.ps1
```

By default, the script uploads everything from `./static/`. If you wish to change this behavior, you can use the `-base` flag. The default namespace is `STATIC`. You can change this by using the `-binding` flag.

For binary files, the script uses the `wrangler kv:bulk` utility to upload base64 encoded versions to Workers KV, which automatically decodes them before storing the raw bytes. This ensures that the Worker can serve binary file formats without any special accomodation. Currently, the upload script only supports the following binary file formats:
- jpg
- jpeg
- png
- gif
- webp
- ttf
- woff
- woff2
- eot
- otf
- pdf
- wav
- mp3
- mpeg
- mp4
- zip
- 7z


This is the same list of binary file encodings supported by the Worker. If you wish to add more binary content types, add them to the array `$BinaryExtensions` in [kv-bulk-uplad.ps1](kv-bulk-upload.ps1). Make sure to associate the approriate `Content-Type` header in [libs.rs](src/libs.rs) so that the Worker can serve the new file type (and submit a pull request upstream to add them for everyone!).

The default filename for bulk uploads is the file `kv-bulk.json`, which is included in the .gitignore for that reason. If you wish to change the default, you can do so with the `-outfile` flag. Keep in mind that the result of base64 encoding many large binary files can be very large, so you may want to delete the outfile after uploading.
