# Packaged resources

Everything in this directory is copied into the Tauri Windows bundle by `tauri.conf.json`.

- `THIRD_PARTY_NOTICES.md`: dependency licensing and distribution notice.
- `licenses/MuPDF-AGPL-3.0.txt`: complete license text copied from the pinned MuPDF source.
- `native/windows-x64/`: reserved for native runtime DLLs if a future dependency is dynamically linked.

MuPDF is currently linked statically, so `native/windows-x64/` intentionally contains no DLL. Build-only inputs (`mupdf` source and `libclang.dll`) belong in `../native/` and are excluded from the installer.
