fn main() {
    let (product_version, build_serial) = release_identity();
    println!("cargo:rustc-env=ARCHETYPES_PRODUCT_VERSION={product_version}");
    println!("cargo:rustc-env=ARCHETYPES_BUILD_SERIAL={build_serial}");
    embed_windows_icon(&product_version, &build_serial);
}

fn release_identity() -> (String, String) {
    let version_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("installer")
        .join("version.json");
    println!("cargo:rerun-if-changed={}", version_path.display());
    let contents = std::fs::read_to_string(&version_path).unwrap_or_else(|error| {
        panic!("required release identity {} could not be read: {error}", version_path.display())
    });
    let field = |name: &str| {
        let marker = format!("\"{name}\"");
        let start = contents.find(&marker).unwrap_or_else(|| {
            panic!("required release identity field {name} is missing from {}", version_path.display())
        });
        let remainder = &contents[start + marker.len()..];
        let value_start = remainder.find(':').unwrap_or_else(|| {
            panic!("release identity field {name} is malformed")
        }) + 1;
        remainder[value_start..]
            .trim_start()
            .trim_matches(|character: char| character == '\"' || character == ',' || character.is_whitespace())
            .split(|character: char| character == '\"' || character == ',' || character.is_whitespace())
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| panic!("release identity field {name} is empty"))
            .to_owned()
    };
    (field("product_version"), field("build_serial"))
}

fn embed_windows_icon(product_version: &str, build_serial: &str) {
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
            let file_version = format!("{product_version}.{build_serial}");
            res.set("FileVersion", &file_version);
            res.set("ProductVersion", &file_version);
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
