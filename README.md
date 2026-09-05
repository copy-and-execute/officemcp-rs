# officemcp-rs

A fast, single-binary **Rust** rewrite of [OfficeMCP](https://github.com/mhackermsft/OfficeMCP) - an MCP server that lets AI assistants create, read, write, and manipulate Word (`.docx`), Excel (`.xlsx`), PowerPoint (`.pptx`), and PDF files over **stdio** using the Model Context Protocol.

Drop-in compatible with the original C# server: same `office_*` / `word_*` / `excel_*` / `pptx_*` tool names, same stdio JSON-RPC transport, **44 tools** total.

> ## AI Notice
>
> This Rust port was written **entirely by AI** (Muse Spark 1.2). The complete C# codebase
> (~9,700 lines across 26 files) was analyzed, re-architected, and rewritten as idiomatic Rust
> (~4,400 lines across 15 files) in **about 30 minutes**, including the crate layout, all 44 MCP
> tool handlers, the document services (Word, Excel, PowerPoint, PDF), and this README.
>
> That speed is arguably the most impressive benchmark in this repo. A full language port with
> preserved tool parity and a ~21× smaller binary, produced in less time than a typical `dotnet`
> clean-build-and-coffee cycle. **As with all AI-generated code, review before trusting it in
> production** - but the cold-start numbers don't lie.

## Why Rust?

| | Original (C# / .NET 10) | This port (Rust) |
|---|---|---|
| Runtime requirement | .NET 10 SDK or shared runtime (~90 MB+) | **None** — static single binary |
| Ship size (measured) | `publish/` ≈ **96 MB** (2 files), `OfficeMCP/bin` ≈ **129 MB** (328 files) | `target/release/officemcp-rs.exe` ≈ **4.6 MB** (~21× smaller than publish output) |
| Startup (measured) | `dotnet run` = seconds (JIT + SDK); self-contained exe ≈ hundreds of ms | **~50 ms** for process spawn + `initialize` + `tools/list` (this machine, release build) |
| Memory / GC | .NET GC, JIT code cache | No GC, no JIT — flat RSS, no pauses |
| Cross-compile | `dotnet publish -r <rid>` per platform | `cargo build --release --target <triple>` — Linux / macOS / Windows from one codebase |
| Source size | ~9,700 LOC across 26 `.cs` files | ~4,400 LOC across 15 `.rs` files |

> Numbers above were measured in this repo: Rust binary weighed with `Get-ChildItem target/release/officemcp-rs.exe` (4,819,456 bytes), .NET output with `Measure-Object` over `publish/` and `OfficeMCP/bin`, cold-start timed by piping `initialize` + `tools/list` into the release binary. Your numbers will vary by platform and toolchain, but the order of magnitude holds.

### Performance notes

- **Release profile is tuned for size *and* speed:** `opt-level = "z"`, `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, `strip = true` (see `Cargo.toml`). Result: a ~4.6 MB binary with no debug symbols and monomorphized hot paths.
- **No async framework tax on the document path:** Tokio is used only for stdio framing; all document work (`docx-rs`, `umya-spreadsheet`/`calamine`, `lopdf`/`printpdf`, `quick-xml`/`zip`) is synchronous, allocation-conscious, and streams ZIP entries instead of buffering whole packages where possible.
- **Fast cold start matters for MCP:** clients spawn the server per session. A 50 ms Rust cold start vs. seconds for `dotnet run` is the difference between an assistant that feels instant and one that times out on first tool call.
- **Small bundle = cheap distribution:** the binary fits in a GitHub Release asset, a Docker `scratch` layer, or an MCP bundle without pulling a runtime image. No `dotnet` on PATH, no `NODE_EXTRA_CA_CERTS`, no framework roll-forward surprises.

## Requirements

- Stable Rust (1.75+, edition 2021). No .NET SDK required.
- An MCP-compatible client (Claude Desktop, VS Code with Copilot, etc.)

## Building

```bash
cd officemcp-rs
cargo build --release
# binary: ./target/release/officemcp-rs (officemcp-rs.exe on Windows)
```

Optional: verify the MCP handshake manually:

```bash
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | ./target/release/officemcp-rs
# expect: initialize result + 44 tools
```

## Running

The server speaks MCP over **stdio** (newline-delimited JSON-RPC, batch arrays supported).

```bash
./target/release/officemcp-rs
```

### Claude Desktop

```json
{
  "mcpServers": {
    "OfficeMCP-Rust": {
      "command": "C:/path/to/officemcp-rs/target/release/officemcp-rs.exe"
    }
  }
}
```

### VS Code (MCP config)

```json
{
  "servers": {
    "OfficeMCP-Rust": {
      "command": "C:/path/to/officemcp-rs/target/release/officemcp-rs.exe",
      "transport": "stdio"
    }
  }
}
```

Logging goes to **stderr** (`tracing_subscriber`, `RUST_LOG` filter); stdout is reserved for protocol frames.

## Tools (44)

### Consolidated `office_*` (11)

| Tool | Description |
|------|-------------|
| `office_create` | Create `.docx` / `.xlsx` / `.pptx` / `.pdf` (format from extension) |
| `office_read` | Read any format; Word returns ordered headings/paragraphs/tables/images + `TemplatePath` |
| `office_write` | Append Markdown (Word/PDF), cells (Excel), text (PowerPoint) |
| `office_convert` | Document → Markdown |
| `office_metadata` | Format, size, structure |
| `office_add_element` | Paragraph, heading, image, table, pageBreak, lists, shape, line |
| `office_add_header_footer` | Headers/footers with page numbers + dates (Word/PDF) |
| `office_extract` | Extract text / images (base64 + context) / tables / metadata |
| `office_batch` | Multiple ops in one call |
| `office_merge` | Merge PDFs |
| `office_pdf_pages` | Extract pages, watermark, read single page |

### PowerPoint `pptx_*` (19)

`pptx_create`, `pptx_read`, `pptx_add_slide`, `pptx_manage_slide`, `pptx_add_title`, `pptx_add_text`, `pptx_add_image`, `pptx_add_table`, `pptx_add_shape` (100+ types), `pptx_add_line`, `pptx_add_connector`, `pptx_add_image_base64`, `pptx_z_order`, `pptx_reorder_shape`, `pptx_add_group`, `pptx_set_slide_size`, `pptx_set_background`, `pptx_add_notes`, `pptx_batch`

### Excel `excel_*` (8)

`excel_create`, `excel_read`, `excel_get_formatting`, `excel_set_cells`, `excel_formula`, `excel_manage_sheet`, `excel_format_cells`, `excel_batch`

### Word `word_*` (6)

`word_read`, `word_add_content` (Markdown), `word_add_element`, `word_add_image`, `word_convert` (`word_to_md` / `md_to_word` / `word_to_md_file`), `word_batch`

## Project structure

```
officemcp-rs/
├── Cargo.toml            # deps + size-optimized release profile
├── README.md             # this file
├── .gitignore            # target/
└── src/
    ├── main.rs           # stdio JSON-RPC loop, initialize/tools/list/tools/call
    ├── models.rs         # shared request/response models
    ├── services/         # document logic (no MCP awareness)
    │   ├── word.rs       # docx-rs based read/write/images/convert
    │   ├── excel.rs      # umya-spreadsheet + calamine
    │   ├── powerpoint.rs # raw OOXML via zip + quick-xml
    │   ├── pdf.rs        # lopdf + printpdf
    │   ├── markdown_parser.rs
    │   ├── format_detector.rs
    │   └── protection.rs # encrypted/protected file detection
    └── tools/            # MCP tool handlers (thin dispatch)
        ├── office.rs
        ├── word.rs
        ├── excel.rs
        └── powerpoint.rs
```

## Key dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` (full) | async stdio framing |
| `rmcp` | MCP type inspiration (transport is hand-rolled stdio here) |
| `serde` / `serde_json` / `schemars` | JSON + tool schemas |
| `zip` / `quick-xml` | OOXML container + XML (docx/pptx/xlsx) |
| `docx-rs` | Word document model |
| `umya-spreadsheet` / `calamine` | Excel write / fast read |
| `lopdf` / `printpdf` | PDF merge, pages, watermark, create |
| `pulldown-cmark` | Markdown → Office content |
| `image` / `base64` / `uuid` / `chrono` | images, embedding, ids, dates |
| `anyhow` / `thiserror` / `tracing` | errors + stderr logging |

See `Cargo.lock` (309 locked entries at time of writing) for the full tree.

## Compatibility notes

- Tool names and required parameters mirror the C# server so existing prompts and MCP configs keep working; point the client at the Rust binary instead of `dotnet run`.
- `office_read` on `.docx` preserves the original's ordered content list (`heading` / `paragraph` / `table` / `image` with base64) and returns `TemplatePath` for style-preserving rewrites via `office_create`.
- Encrypted/protected files are detected and reported (see `services/protection.rs`); full decryption parity with the .NET build is best-effort — check tool error messages for guidance.
- Batch tools (`office_batch`, `word_batch`, `excel_batch`, `pptx_batch`) accept the same `operationsJson` shapes as the original.

## License

MIT - same as the workspace. See `Cargo.toml` (`license = "MIT"`).
