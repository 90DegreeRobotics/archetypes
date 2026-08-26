# Building a NeuroCognica Windows Product, End to End

**Written 2026-08-25 by the agent that took EOAI-MGS from a Tkinter shell to a
signed, installed, witnessed 0.6.0, and then did the same audit and repair pass
on NC Company Database through to a signed 0.2.1.**

This is a handoff. It is written for the next agent — specifically the one
picking up `C:\archetypes` — and it is deliberately prose rather than a
checklist, because the checklist is the easy half and the judgement is the half
that gets lost. Everything below was either done by hand in this session or read
directly off the machine. Where something has not been proven, it says so.

---

## 1. What you are actually being asked to produce

Not "a build". The deliverable is a **Windows lifecycle executable**: a single
signed installer a person can download onto company hardware, run without an
admin prompt, and get a working product from — which then appears correctly in
the Start Menu, on the desktop, and in Add/Remove Programs with real metadata,
and which uninstalls cleanly and leaves nothing behind except the user's own
data.

Every one of those clauses is a gate that has failed before. The install that
puts the exe down but registers no uninstaller. The uninstaller that removes the
program but leaves a dead Start Menu folder. The shortcut carrying the previous
version's icon because Windows cached it. The "signed" release that shipped
unsigned because the build script treated a signing failure as a warning. You
will be measured on the whole lifecycle, not on whether `cargo build` succeeded.

The governing rule from the build doctrine, which is worth memorising: **a green
check with no artifact behind it is a lie, and an artifact nobody opened is not
evidence.** If you claim the app runs, you screenshotted the running app. If you
claim it is signed, you ran `Get-AuthenticodeSignature` and read `Valid`.

---

## 2. Read these first, in this order

1. `C:\corpus\THE_CHARTER_OF_COGNITIVE_SOVEREIGNTY.md` — supreme law.
2. `C:\chronos2\docs\NEUROCOGNICA_BUILD_DOCTRINE.md` — how Michael builds and
   ships. Section 15 covers everything a Windows buyer sees.
3. `C:\chronos2\docs\windows\SIGNING.md` — the signing front door, and the
   incident history that explains why signing is not optional.
4. `C:\EOAI-MGS\PLAN_2026-08-25_VISUAL_PRODUCT_SYSTEM.md` — the product visual
   system across all seven products.
5. `C:\archetypes\AGENTS.md` and `CLAUDE.md` — the repo's own contract.

When two documents disagree, the later one wins **and you fix the loser in the
same commit**. A dated document is a claim about its date, not about today.
Michael's instruction in the live session outranks every document here.

---

## 3. What Archetypes is right now, and what that changes

I looked. Do not assume it resembles the two products I just shipped.

Archetypes is a **Rust workspace**, not a Python app. `Cargo.toml` declares two
members, `crates/engine` and `crates/launcher`, producing `engine.exe` and
`launcher.exe`. There is a `dist\` folder containing those binaries plus
`archetypes.ico`, assets, help, scripts and speech. There are already
`scripts\install_product.ps1`, `install_shortcut.ps1`, `uninstall_product.ps1`
and `setup_windows.ps1`.

What it does **not** have: an Inno Setup script, a `build.ps1`, any signing
whatsoever, or a single-file installer. Its current shape is a loose directory
plus install scripts. Your job is to close that gap.

This matters because the two products I shipped are PyInstaller-based, and
roughly a third of what I did does not transfer. **Do not copy the PyInstaller
half.** What transfers is: the Inno Setup layer, the signing layer, the version
discipline, the icon and shell grammar, the witness discipline, and the repo
hygiene. What does not transfer is the `--onefile` spec, the `_MEIPASS` resource
resolution, and the `--exclude-module cefpython3` line.

Rust actually makes your life easier here: `cargo build --release` gives you
real native binaries with no bootloader, no bundled interpreter, and no
`_MEIPASS` path juggling. Sign the binaries directly, then feed them to Inno.

Also note the git remote is `90DegreeRobotics/archetypes`, not `NeuroCognica`.
Both owners are under the NeuroCognica banner; that is expected, not an error.

---

## 4. Signing: the part that is not negotiable

**Every versioned executable is Authenticode-signed on compile.** This is
standard protocol, not a release-day extra, and it is never a thing to ask
permission for. Michael is a verified author in the Microsoft Windows ecosystem
and signs large numbers of versioned executables as normal practice.

The reason is concrete and documented in `SIGNING.md`: Windows Defender has
**actively quarantined** unsigned NeuroCognica installers — build 1.2.0 on
2026-06-21 flagged as `Trojan:Win32/Bearfoos.B!ml`, and build 2.0.32 on
2026-08-16 on this same machine, real-time protection on, no exclusion set.
Smart App Control hard-blocks them outright. A buyer can pay and never get the
file to run. An unsigned build is not a build; it is a support ticket.

### How it works on this machine

Azure Artifact Signing. There is **no `.pfx` on disk, no password, and no secret
in any repo** — Microsoft holds the private key in a FIPS 140-3 Level 3 HSM, and
`signtool` reaches the service through a loader DLL that authenticates as
whoever `az login` established. You will never handle a credential, which is
exactly the point.

The shared assets live once, outside every product repo:

- dlib: `C:\chronos2\tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll`
- metadata: `C:\chronos2\installer\signing\azure-metadata.json`
  (endpoint `https://eus.codesigning.azure.net/`, account `chronosophiasigning`,
  profile `chronosophiapublic`)
- Azure CLI: `C:\Program Files\Microsoft SDKs\Azure\CLI2\wbin`

**The dlib must be the `bin\x64\` copy.** The NuGet package unpacks the loader
into an architecture subdirectory and there is no copy at the package root.
Pointing at the root gives you a *silently unsigned installer*, because the
build script treats a missing dlib as a yellow warning. That exact mistake sat
in the documentation for four days.

The environment contract, `NEUROCOGNICA_*` first with `CHRONOS_*` accepted as
fallback:

```
NEUROCOGNICA_SIGNING_MODE=azure
NEUROCOGNICA_AZURE_SIGNING_DLIB=C:\chronos2\tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll
NEUROCOGNICA_AZURE_SIGNING_METADATA=C:\chronos2\installer\signing\azure-metadata.json
NEUROCOGNICA_SIGNING_TIMESTAMP=http://timestamp.acs.microsoft.com/
```

Copy `templates\installer\signing_env.ps1` into the repo and dot-source it. Do
not retype these paths from memory in a shell; that is how a build silently goes
unsigned.

### The gate that actually matters

I want to be blunt about this because it is the single easiest way to ship a lie.
`build.ps1` in azure mode treats a missing dlib **or a failed signature** as a
yellow warning and **still emits an installer**. You will get a green-looking
build and an unsigned artifact.

So the pattern I settled on, and which you should copy from
`templates\installer\build_signed.ps1.template`, brackets the build with two
gates:

1. **Before**: run `scripts\check_signing_ready.ps1`. Exit 0 means a signed
   build is possible. It checks signtool, the dlib, the metadata file, the
   metadata *contents* (an unfilled template fails here rather than becoming a
   confusing signtool error later), Azure authentication, and the timestamp
   authority. It changes nothing.
2. **After**: run `Get-AuthenticodeSignature` on every artifact and **throw**
   unless each reports `Valid`. Expect subject
   `CN=Michael Holt, O=Michael Holt, L=Normal, S=il, C=US`, issuer
   `CN=Microsoft ID Verified CS EOC CA 03`.

Make `build_signed.ps1` the only entry point anyone is told to use. Leave
`build.ps1` in place as the mechanism, but never document it as the way to cut
a release.

One thing that will look alarming and is fine: the certificate's `NotAfter` is
only a few days out. Azure Artifact Signing issues short-lived certificates by
design. The RFC-3161 countersignature from the timestamp authority is what keeps
the signature valid after the certificate expires — which is why timestamping is
mandatory, not optional.

Sign **both** the product binaries and the installer, and set
`SignedUninstaller=yes` in the Inno script. An unsigned uninstaller produces a
SmartScreen warning months later when the user finally removes the product, and
by then nobody remembers why.

---

## 5. The visual system

All seven NeuroCognica products are meant to read as **one operating system with
different instruments inside it — same body, different flame**. The accent
colour identifies the product; it does not license a different design language.
Archetypes is **green, `#22c55e`**, on a dark neutral base.

### Icons: mark-only, plate in the product colour

The mark is six white dots around one, on a rounded plate filled with the
product colour. Two rules, both of which have already cost real time:

**The app icon is mark-only.** No wordmark text inside a taskbar icon — it turns
to mud below 32px. EOAI-MGS shipped a wordmark lockup on a transparent
background; at taskbar size it was an unreadable smudge that disappeared
entirely against dark chrome. I regenerated it as a mark-only red plate and the
difference is immediate.

**The plate is opaque and carries the colour; the dots stay white.** A
transparent-background mark vanishes on whichever taskbar theme you did not test
against. Michael caught this one himself on the blue set: *"the blue should be
the black and the white text and dots remain white."* That sentence is the rule.

Do not generate icons by hand. Run:

```powershell
python C:\NeuroCognica_Brand\scripts\build_product_brand.py archetypes
```

That writes `C:\NeuroCognica_Brand\products\archetypes\` containing
`app_icon.ico` (all seven Windows sizes: 16/24/32/48/64/128/256),
`app_icon_256.png`, `favicon_512.png`, `ui_brand_mark.png`,
`installer_welcome.png`, `installer_small.png`, `document_mark.png`, and
`tokens.css`. Copy what you need into the repo's `assets\` and note in the
README where it came from. The generator stays in the kit; the repo carries only
generated output.

Archetypes currently ships its own `dist\archetypes.ico`. Replace it. Verify the
result yourself at 16px — open it, look at it, do not trust the file listing.

### Install wizard branding

Inno's default wizard is grey and generic and instantly reads as amateur
software. The kit generates the two images Inno wants: `installer_welcome.png`
for `WizardImageFile` (the 164x314 left rail, rendered at 2x so it stays crisp
through 200% Windows scaling) and `installer_small.png` for
`WizardSmallImageFile` (the 58x58 header mark). Both carry the product plate,
the NeuroCognica wordmark, the product name in the accent colour, and the
product line beneath it. Wire them in the `[Setup]` section and set
`WizardStyle=modern`.

### The application shell

If Archetypes grows a management or configuration surface, it should be the same
shell as its siblings: a local HTML/CSS/JS surface in a native desktop window —
no browser chrome, no console — with a fixed left navigation rail, a brand block
at the top of the rail, a large search/action bar across the top of the work
area, glass panels for repeated data and controls, and toasts in the lower
right. Copy `templates\ui\style.css` and replace only its `:root` block with the
product's `tokens.css`. That is the entire intended difference between products.

Do not build a marketing landing page inside the app. The first screen is the
working tool.

Two layout traps I hit and fixed in both products, which you should simply never
introduce:

- **No `zoom` on `body`.** It makes `100vw`/`100vh` resolve to a fraction of the
  real window and leaves dead bands down the right and along the bottom. NC
  Company Database was rendering at 86% of its window because of this.
- **No fixed pixel canvas widths.** NC had `width: 1560px` in the stylesheet and
  a `1100px !important` override fighting it from an inline `<style>` block. The
  result was that on Home, three of six metric cards and *every action card*
  were clipped out of view — the main screen of a shipping product had zero
  clickable actions and nobody had noticed. Let the layout follow the window.

### Honesty rules in the UI

These are Michael's standing product rules and he has stated them directly:

- **No CLI-only operational path.** If a capability exists, it needs a
  buyer-facing button that is fully wired to real behaviour.
- **Ship an empty database.** No demo records, no sample rows, no fake activity.
  Empty state is honest state.
- **No theatre.** A control that is visible must do something. I found NC
  shipping two buttons — Refresh and Sync GitHub — that were in the DOM, fully
  wired in JavaScript, and hidden by a `display: none` rule. Dead UI in both
  directions is a defect.

---

## 6. The end-to-end procedure

Here is the shape of the work. Adapt the compile step to Rust; the rest holds.

**Version first.** A single source of truth for the version string, referenced
everywhere else. Bump it *before* you build, never after. I learned this the
irritating way: I rebuilt NC Company Database with substantial UI fixes but left
the version at 0.2.0, which meant the installer filename, the Add/Remove
Programs entry and the in-app footer were all identical to the broken build.
There was no way for the operator to tell which one they were running. Bumping
to 0.2.1 was the fix and it should have been step one.

**Compile.** `cargo build --release`. Confirm the binaries are the Windows GUI
subsystem where appropriate — a console window flashing up on launch is a defect
for a buyer-facing product.

**Sign the binaries.** Before they go into the installer.

**Package.** Inno Setup, from `templates\installer\product_setup.iss.template`.
Per-user install: `PrivilegesRequired=lowest`, `DefaultDirName` under
`{localappdata}\NeuroCognica\<Product>\App`. No admin prompt, no Program Files.
Generate the `AppId` GUID **once** and never change it — it is what lets a later
version upgrade in place rather than installing alongside itself.

**Sign the installer and the uninstaller.**

**Verify.** Both artifacts `Valid`, and record the SHA-256 of each in
`release.json` alongside the version, git commit, and build time. The release
metadata must be honest: if something went unsigned, it says unsigned.

**Install it, for real.** Then check, and write down what you saw:

- install root contents, and installed-exe hash parity with the built artifact
- Start Menu, Desktop and uninstall shortcuts — target, arguments, working
  directory, and **icon path**
- the uninstall registry key: DisplayName, DisplayVersion, Publisher,
  InstallLocation, DisplayIcon, UninstallString, QuietUninstallString,
  EstimatedSize, InstallDate, NoModify, NoRepair, SystemComponent,
  WindowsInstaller
- that it enumerates in Add/Remove Programs under the right name and version

**Launch it from the Start Menu shortcut**, not from the exe path. That is the
buyer's path and it is the one that exercises the shortcut metadata.

**Screenshot the running app.** Details in section 7 — this is fiddlier than it
sounds.

**Uninstall, and verify the cleanup**: install root, Start Menu folder, Desktop
shortcut, current *and legacy* registry keys. Leave the shared
`%LOCALAPPDATA%\NeuroCognica` parent alone — six sibling products live under it.

**Commit, push, and prove the push.** Confirm local `HEAD`, `origin/main` and
the remote ref all match. Branch is `main`; there is one branch.

---

## 7. Witnessing, and the two traps in it

You will need screenshots, and this is where I wasted the most time, so take
this section literally.

**`PrintWindow` produces artifacts on glass.** Capturing a window directly via
`PrintWindow` with `PW_RENDERFULLCONTENT` works and captures regardless of
z-order — but on panels using `backdrop-filter` blur it renders garbage. It
convinced me for several minutes that NC's People tab had a missing table and
clipped, overflowing buttons. It had neither. A DOM probe and a plain screen
grab both showed a correct page. If a screenshot shows something structurally
bizarre, verify it against the DOM before you report it as a defect.

**`SetForegroundWindow` fails silently from a background process.** Windows
refuses it unless the calling thread shares input state with the foreground
thread. My grab came back showing an entirely different application. The fix is
`AttachThreadInput` around the call, and then — this is the important part —
**assert that the target window is actually in the foreground and refuse to save
the file if it is not.** A misleading witness is worse than no witness.

For inspecting a webview's real layout, drive the bridge: `window.evaluate_js()`
returning `getBoundingClientRect()` and `getComputedStyle()` for the elements you
care about gives you exact numbers and settles arguments a screenshot cannot.

Downscale and quantize witness PNGs before committing — 1280px wide, 128
colours, which took mine from ~2.3 MB to ~150-200 KB each with no loss of
legibility. Screenshots at native 4K resolution do not belong in a repo.

Also, be careful about what else is on the screen. One of my screen grabs caught
an NVIDIA driver installer that had popped up mid-capture and briefly convinced
me the app had zoomed to 300%. It had not; the driver install had flashed the
display. Michael spotted it before I did.

---

## 8. Repo hygiene

Repos are compilation sources. They stay tight, cloneable, and fastidiously
controlled, and a clone plus the documented build recipe must be sufficient to
reproduce the product.

**No generated artifact enters git.** No exe, no installer, no database, no
model weights, no cache, no build directory. I found EOAI-MGS tracking a 42 MB
installer as source, which meant every rebuild dirtied the tree with a binary
diff. `installer/output/` went into `.gitignore` and the tracked copies were
removed with `git rm --cached`, which keeps the files on disk while untracking
them.

**Delete unreachable code rather than letting it imply capability.** NC carried
an 894-line Tkinter application that nothing imported, no entry point invoked,
and the packaging spec did not reference — while the README described it as a
"source fallback". I wired it behind an explicit `--classic` flag, which cost
almost nothing and made the documentation true. Either wire it or delete it; git
history retains what you remove. What you must not do is leave dead code sitting
there reading as a feature. NC also had a `_split_pages` function and its
constant that were never called from anywhere; those I simply deleted.

**Write tests that hold the promises which rot silently.** The most valuable
thing I added to both repos was a structural test file. It parses the HTML and
the JavaScript and asserts that every `<button id>` is bound to a handler, and
that every `api().method()` call resolves to a real method on the Python bridge.
Those two checks make theatre impossible to ship. Alongside them: assertions
that the layout has not regressed to a fixed canvas, that no control is hidden
by CSS, that the icon carries all seven sizes and is the right colour, and that
`cefpython3` is not installed in the build environment. Copy
`templates\tests\test_product_shell.py.template` and adapt it.

**Keep a truth file.** EOAI-MGS has `TRUTH_AUDIT.md`, which separates what is
proven (naming the test that holds it), what is scaffolded, and what was
removed. It is the most useful document in either repo, because it is where you
record the things that are *not* true yet. When I measured the offline corpus
mirror and found it landing 23 of 45 sources — the rest refused by bot
protection on `ready.gov`, `fema.gov`, `fcc.gov` and others — the honest move
was to record the measured ceiling and the reason, not to quietly report success.

---

## 9. Things I got wrong, so you do not repeat them

- I called a capture artifact a product defect and had to retract it. Verify
  structural claims against the DOM, not against a screenshot.
- I rebuilt a product without bumping its version, producing an installer
  indistinguishable from the broken one it replaced.
- I rebuilt with `-SkipSign` and overwrote a previously **signed** installer in
  the operator's Downloads folder with an unsigned one. Michael had to ask
  whether the file was current. Signing is protocol; skipping it needs a reason
  and an announcement, and honestly should just never happen.
- I initially assumed a User-Agent change would fix HTTP 403s from government
  sites. It made things worse — pages that had been succeeding started
  refusing. Test the hypothesis before building on it, and when the blocker
  turns out to be bot protection, stop rather than escalate.

---

## 10. Definition of done for Archetypes

You are finished when all of the following are true and you have personally
seen the evidence for each:

1. `cargo build --release` produces the product binaries, GUI subsystem where
   the user sees them.
2. Icons come from the brand kit, green plate, white dots, mark-only, all seven
   sizes, and you have looked at the 16px rendering.
3. The Inno wizard shows NeuroCognica branding, not Inno's default grey.
4. One signed installer exists, `PrivilegesRequired=lowest`, stable `AppId`.
5. Binaries, installer and uninstaller all report Authenticode `Valid` under
   `CN=Michael Holt`, timestamped.
6. `release.json` records version, commit, build time, and both SHA-256 hashes
   honestly.
7. A real install has been performed and verified: files, hash parity,
   shortcuts with correct icons, complete uninstall registry, Add/Remove
   Programs enumeration.
8. The app has been launched **from the Start Menu shortcut** and screenshotted.
9. A real uninstall has been performed and cleanup verified, with the shared
   NeuroCognica parent directory left intact.
10. No generated artifact is tracked in git; a fresh clone plus the documented
    recipe reproduces the build.
11. `STATUS.md` and the truth file say what is proven, what is not, and what was
    measured — including anything that did not work.
12. Local `HEAD`, `origin/main` and the remote ref all match.

If you cannot complete one of these, say so plainly and say why. A truthful
partial result is worth more here than a confident complete-sounding one, and it
is the thing this operator actually rewards.
