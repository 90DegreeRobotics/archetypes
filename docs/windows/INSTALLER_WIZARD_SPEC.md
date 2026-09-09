# Installer and Uninstaller Wizard Specification

**Canonical Reference for the Archetypes Repository**  
*Borrowing and dressing the Inno Setup architecture from `C:\chronos2\installer\chronos_setup.iss`*

---

## 1. Architectural Heritage & Principles

The Archetypes installation system borrows the battle-tested Inno Setup 6 engine and architecture from `C:\chronos2`, preserving its rigorous engineering standards while dressing it in a bespoke dark-fantasy aesthetic tailored for the game.

### Core Non-Negotiable Standards Borrowed from `chronos2`:
1. **DPI-Safe Dynamic Layout:**
   - Wizard layout strictly obeys high-DPI displays (100%, 125%, 150%, 200%).
   - All custom controls use `ScaleX()` and `ScaleY()`.
   - Progress bars anchor to the true bottom of the page surface to prevent the classic Inno 6 bug where progress bars clip out of view on high-res monitors.
   - Text elements enforce `WordWrap = True` to prevent label truncation.

2. **Smart App Control (SAC) Compliance:**
   - Employs compile-time signing (`SignTool=archetypes`) so that `ISCC` signs:
     - `uninst.e32.tmp` (the uninstaller),
     - The runtime temporary self-copy extracted to `%TEMP%` (`Setup.e32`), and
     - The outer setup binary.
   - Eliminates Windows Error 4551 ("An Application Control policy has blocked this file").

3. **Single-Instance Mutex:**
   - `SetupMutex=Archetypes_Setup,Global\Archetypes_Setup` prevents concurrent setup instances from corrupting game files.

4. **Graceful Cancellation:**
   - `AllowCancelDuringInstall=yes` ensures players can safely cancel, press ESC, or close the wizard.

---

## 2. Dressing the Wizard for Archetypes (Game Aesthetic & Branding)

Where `chronos2` uses a corporate deep-slate technical palette, Archetypes dresses the wizard in a rich, ceremonial, dark fantasy visual theme reflecting the Castle Rotunda and the Inner Chambers of the Seven Archetypes:

### 2.1 Color Palette
- **Deep Obsidian (Wizard Background):** `#06050C`
- **Chamber Slate (Dynamic Dark Surface):** `#0E0C1A`
- **Mystic Gold (Accent & Borders):** `#E2C374` (matching the torchlight and the Council Table rim)
- **Portal Violet (Secondary Glow):** `#6B4CE6` (matching the rotating Stargate vortex)
- **High-Contrast Text (Light Mode / Dialogs):** `#F4F2FA`

### 2.2 Visual Assets & Branding Artwork
| Asset | Source / Dimensions | Role |
|---|---|---|
| `WizardImageFile` | `assets\branding\installer_welcome.png` (328 × 628 px @ 2x) | Vertical welcome banner displaying the grand torchlit Castle Rotunda, vaulted stone ceiling, and the Council Table with the seven archetype thrones. |
| `WizardSmallImageFile` | `assets\branding\installer_small.png` (110 × 110 px @ 2x) | Header seal displayed in the upper right: the Archetypes ceremonial crest in burnished gold. |
| `SetupIconFile` | `assets\branding\archetypes.ico` | Multi-resolution icon (16x16 to 256x256) embedded in the installer executable. |
| `UninstallDisplayIcon` | `{app}\assets\branding\archetypes.ico` | Displayed in Windows "Installed Apps" / Control Panel. |

---

## 3. Directory Layout & Storage Law

The installer strictly enforces the separation between immutable application binaries and mutable user/save data:

```
Immutable Installation Directory:
  C:\Program Files\Archetypes\   ({autopf}\Archetypes)
    ├── engine.exe               (Main game runtime)
    ├── launcher.exe             (Windows desktop launcher)
    ├── assets\                  (Game assets, GLB models, shaders, textures)
    │     ├── scenes\table.glb   (Council Table & Stargate Portal)
    │     ├── textures\          (Stone, marble, gold materials)
    │     └── audio\             (Soundtrack and spatial ambiance)
    ├── docs\                    (Game manual and lore codex)
    └── unins000.exe             (Digitally signed uninstaller)

Mutable Player Data Directory (PRESERVED ON UNINSTALL):
  %LOCALAPPDATA%\NeuroCognica\Archetypes\
    ├── saves\                   (Player game progress, chamber unlocks)
    ├── profiles\                (Player archetype alignment and stats)
    ├── config\                  (Graphics, display, audio, keybind settings)
    └── logs\                    (Runtime engine diagnostics)

Machine-Wide Cache (Optional):
  %ProgramData%\NeuroCognica\Archetypes\
    └── cache\                   (Shader pipeline cache, precompiled PSO)
```

---

## 4. Install Flow & User Experience

1. **Welcome Screen:**  
   Branded banner introduces the player to the Seven Archetypes.
2. **License / Lore Codex Agreement (`EULA.txt`):**  
   Player-centric software agreement granting personal entertainment and modding rights.
3. **Pre-flight & System Requirements (`DEPENDENCY_NOTICE.txt`):**  
   Discloses graphics API requirements (Vulkan 1.3 / DirectX 12, 64-bit Windows 10/11) and audio devices.
4. **Destination Directory:**  
   Defaults cleanly to `C:\Program Files\Archetypes` (requires standard UAC elevation).
5. **Component & Task Selection:**
   - `[X] Core Game Engine & Castle Rotunda` (Mandatory)
   - `[X] High-Resolution 3D Chamber Models` (Default checked)
   - `[X] Ambient Orchestral Soundtrack` (Default checked)
   - `[X] Create Desktop Shortcut` (`Archetypes.lnk`)
   - `[X] Pin to Start Menu` (`NeuroCognica\Archetypes`)
6. **Installation Progress:**  
   Smooth progress indicator displaying evocative stage updates:
   - *"Carving stone rotunda and buttress pillars..."*
   - *"Summoning the animated Council Table..."*
   - *"Weaving procedural torchlight and volumetric fog..."*
   - *"Registering Windows desktop shortcuts..."*
7. **Finish Screen:**  
   Provides a clean "Launch Archetypes" checkbox to immediately run the game without needing to locate shortcuts manually.

---

## 5. Graceful Uninstall Contract

Uninstalling Archetypes must be clean, respectful, and safe:

1. **What Is Removed:**
   - The entire immutable program tree (`C:\Program Files\Archetypes\**`).
   - Desktop and Start Menu shortcuts (`Archetypes.lnk`).
   - Windows Registry registration keys (`HKLM\SOFTWARE\NeuroCognica\Archetypes`).

2. **What Is Strictly PRESERVED:**
   - **Player Save Files:** `%LOCALAPPDATA%\NeuroCognica\Archetypes\saves\**` is NEVER deleted.
   - **Player Configurations & Lore Progress:** All player choices and unlocked chambers are preserved so reinstalling or upgrading never destroys progress.
   - **Shared Runtimes:** DirectX, Vulkan, and Visual C++ redistributables are never removed.

3. **Uninstall Receipt:**
   Upon completion, the uninstaller writes an informative text receipt:
   `%LOCALAPPDATA%\NeuroCognica\Archetypes_uninstall_receipt.txt` confirming what was removed, what was preserved, and where player save files remain.
