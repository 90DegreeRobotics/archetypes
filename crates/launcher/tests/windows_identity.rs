//! Archetypes is a NeuroCognica product on Windows and has to look like one.
//!
//! Before 2026-08-25 it wrote three top-level Start Menu shortcuts
//! (`Archetypes`, `Archetypes Help`, `Uninstall Archetypes`), pointed the Start
//! Menu entry at `C:\archetypes\dist\launcher.exe` - the developer checkout -
//! while the product was installed under `%LOCALAPPDATA%\Programs\Archetypes`,
//! and rasterised a cyan spider glyph into a single-size `.ico` on every restage,
//! overwriting the repo copy so the real mark could not be committed.
//!
//! The rule: `C:\NeuroCognica_Brand\docs\START_MENU_FAMILY.md`. One flat `.lnk`
//! in `Programs\NeuroCognica`, wearing the green NeuroCognica family mark at all
//! seven Windows sizes.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Number of images in an `.ico` directory: reserved(2) type(2) count(2).
fn ico_image_count(relative: &str) -> u16 {
    let path = repo_root().join(relative);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert!(bytes.len() > 6, "{} is not an icon", path.display());
    assert_eq!(
        (bytes[2], bytes[3]),
        (1, 0),
        "{} is not an icon resource",
        path.display()
    );
    u16::from_le_bytes([bytes[4], bytes[5]])
}

#[test]
fn product_icon_carries_every_windows_size() {
    // A single 256px face is a defect even when it looks right: Windows
    // downscales it for the taskbar instead of using a tuned 16px face.
    assert_eq!(
        ico_image_count("assets/icons/archetypes.ico"),
        7,
        "the product icon must carry 16/24/32/48/64/128/256"
    );
}

#[test]
fn the_product_icon_is_not_redrawn_at_stage_time() {
    let staging = read("scripts/install_shortcut.ps1");

    assert!(
        !staging.contains("GetHicon"),
        "staging is generating an icon again instead of copying the committed family mark"
    );
    assert!(
        !staging.contains("architect-icon.png"),
        "staging is rasterising the old glyph over the family mark again"
    );
    assert!(
        staging.contains("assets\\icons\\archetypes.ico"),
        "staging must copy the committed product icon"
    );
}

#[test]
fn installs_write_one_flat_shortcut_in_the_family_folder() {
    for script in ["scripts/install_product.ps1", "scripts/install_shortcut.ps1"] {
        let body = read(script);
        assert!(
            body.contains("neurocognica_start_menu.ps1"),
            "{script} does not use the shared family Start Menu rule"
        );
        assert!(
            body.contains("Remove-NeuroCognicaLegacyShortcut"),
            "{script} leaves the old top-level shortcuts in place on upgrade"
        );
        assert!(
            !body.contains("Archetypes Help.lnk"),
            "{script} creates a second Start Menu entry; help ships inside the install"
        );
        assert!(
            !body.contains("Uninstall Archetypes.lnk"),
            "{script} creates an uninstall shortcut; uninstall is Add/Remove Programs"
        );
    }
}

#[test]
fn the_family_helper_never_removes_the_shared_folder() {
    let helper = read("scripts/neurocognica_start_menu.ps1");

    assert!(helper.contains("Start Menu\\Programs\\NeuroCognica"));
    assert!(
        !helper.contains("Remove-Item -LiteralPath $FamilyDir"),
        "removing the family folder would take ChronoSophia2 and the siblings with it"
    );
}

#[test]
fn uninstall_removes_this_product_and_its_legacy_names() {
    let body = read("scripts/uninstall_product.ps1");

    assert!(body.contains("Get-NeuroCognicaShortcutPath"));
    assert!(body.contains("Remove-NeuroCognicaLegacyShortcut"));
}
