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
        if icon.is_file() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(icon.to_string_lossy().as_ref());
            if let Err(error) = res.compile() {
                println!("cargo:warning=winres icon embed failed: {error}");
            }
        } else {
            println!(
                "cargo:warning=missing {} — Windows exe icon not embedded",
                icon.display()
            );
        }
    }
}
