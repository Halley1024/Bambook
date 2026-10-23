# Native build dependencies

This directory contains build-time native dependencies that are not fetched or located by Tauri at runtime.

## `libclang/`

- Version: `18.1.1`
- Source package: PyPI `libclang==18.1.1`
- Purpose: `mupdf-sys` uses bindgen to generate Rust FFI bindings during compilation.
- Runtime packaging: excluded. The application does not load `libclang.dll` after it has been built.

On Windows hosts, `mupdf-sys/build.rs` automatically sets `LIBCLANG_PATH` to the sibling `libclang/clang/native` directory when `libclang.dll` exists there. An existing `LIBCLANG_PATH` takes precedence. If the bundled DLL is absent, the normal system libclang search is used. Linux hosts always use the normal system search unless `LIBCLANG_PATH` is explicitly set. No `.cargo/config.toml` is required.

To restore the directory on a new checkout when native binaries are not stored in source control:

```powershell
python -m pip install libclang==18.1.1 --target src-tauri/native/libclang --no-compile
```

## MuPDF

- `mupdf-rs/`: vendored `mupdf` Rust safe wrapper, version `0.8.0`.
- `mupdf-sys/`: vendored FFI crate and pinned MuPDF C source, version `0.8.0`.

`mupdf-rs/Cargo.toml` points to `../mupdf-sys`, so Cargo does not download either MuPDF crate during a clean build. MuPDF is compiled and linked statically. No MuPDF DLL is required beside the final executable, which avoids DLL search-path and version mismatch issues in the NSIS package.

`native/` is deliberately not listed in `tauri.conf.json > bundle.resources`: these files are compiler inputs, not application runtime data. Shipping the source and `libclang.dll` inside every installer would add roughly 150 MB without helping the installed application.

## Clean Windows build

Required system components are Visual Studio 2022 C++ Build Tools and a Windows SDK. From the repository root:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri:build:windows
```

The MuPDF build script locates the bundled `libclang.dll`; no global LLVM installation is required when that DLL is present. The existing build dependencies locate Visual Studio's C/C++ compiler and MSBuild automatically. Ensure Rust's `%USERPROFILE%\.cargo\bin` is on `PATH` so `cargo` and `rustc` can be invoked.
