# Brand assets

- `revelation-logo-original.webp`: the app logo, supplied by the owner (C. Ethan Key) on 2026-10-24. It is 1254×1254 and shows the star and open book above the "Revelation" wordmark.
- `revelation-icon-1024.png`: the app icon. It is the star and book only, with the wordmark removed at the owner's request, centred on a 1024×1024 dark canvas (#080B12) with soft edges. It is made from the original by `make-icon.ps1`, using built-in Windows WPF imaging with no extra tools:

  ```powershell
  powershell -NoProfile -STA -File assets\brand\make-icon.ps1 -Src assets\brand\revelation-logo-original.webp -Out assets\brand\revelation-icon-1024.png
  ```

## Provenance

Recorded in `sources/SOURCES.md`, row **BRAND-LOGO**.

- Supplied by the owner, C. Ethan Key, on 2026-10-24. **Made with ChatGPT**: an AI-generated image. The only later changes are the mechanical crop, scale, centring and soft edges done by `make-icon.ps1`.
- SHA-256: original `2bdbece3e616643eb4d4cf98343dd39a0432958d86d09c4dbfdf2c9b05936712`; icon `40b8b2195b8816d3d514c0580ae97962cf008e5098c77dc0cab4951e230bb4b8`; script `65f2c126cde1693c0433d251626c28622cdc4e65756e4ec4400f1bb883007661`. A changed file needs the row updated. The WebP has no C2PA or other provenance metadata left in it.
- These files are **not** under the project's MIT licence; see [`LICENSE`](LICENSE) in this directory and the licence section of the root README. The same artwork, in derived sizes, is also tracked in `app/public/brand/` (three PNGs) and `app/src-tauri/icons/` (the app icon set); the exclusion covers those copies too.
