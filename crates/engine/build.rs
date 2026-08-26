fn main() {
    embed_windows_icon();
}

fn embed_windows_icon() {
    #[cfg(target_os = "windows")]
    {
        let icon = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
            .join("icons")
            .join("archetypes.ico");
        // Without this, cargo never re-runs the build script when the icon
        // changes, so a corrected .ico leaves a stale mark embedded in the exe
        // and the fix looks like it did nothing. That is exactly what happened
        // when the NeuroCognica family mark replaced the old glyph.
        println!("cargo:rerun-if-changed={}", icon.display());
        if icon.is_file() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(icon.to_string_lossy().as_ref());
            res.set("FileVersion", "0.3.0.0");
            res.set("ProductVersion", "0.3.0.0");
            res.set("ProductName", "Archetypes");
            res.set("FileDescription", "Archetypes — Council Chamber");
            res.set("LegalCopyright", "Copyright (c) Michael Holt / NeuroCognica");
            if let Err(error) = res.compile() {
                println!("cargo:warning=winres embed failed: {error}");
            }
        } else {
            println!(
                "cargo:warning=missing {} — Windows exe icon not embedded",
                icon.display()
            );
        }
    }
}
