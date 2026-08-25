//! Build script: embeds the Windows executable icon.
//!
//! On Windows targets the icon in `assets/icon.ico` is compiled into the
//! `.exe` via `assets/windows/app.rc`, so Explorer and the taskbar show it
//! even before the window exists. On other platforms nothing is compiled
//! here; the icon is embedded in the binary by `include_bytes!` instead
//! (see `src/icon.rs`).

fn main() {
    // `CARGO_CFG_TARGET_OS` reflects the *target*, so this stays correct
    // even when cross-compiling from another host.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/windows/app.rc");
        println!("cargo:rerun-if-changed=assets/icon.ico");
        embed_resource::compile("assets/windows/app.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to embed the Windows application icon");
    }
}
