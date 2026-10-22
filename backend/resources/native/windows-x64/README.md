# Windows x64 runtime native resources

MuPDF is currently linked statically into the application executable, so this directory intentionally contains no MuPDF DLL.

If the project later switches to dynamic MuPDF linking, place the pinned x64 runtime DLLs here and add them to `tauri.conf.json` resources. Never copy `libclang.dll` here: libclang is a build-time dependency only.
