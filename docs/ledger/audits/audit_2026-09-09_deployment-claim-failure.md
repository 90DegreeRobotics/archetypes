# Audit: Deployment Claim Failure and Acceptance-Bar Defect — 2026-09-09

## Scope

This record addresses a buyer-facing failure: an agent previously reported that
the taskbar launch path had been updated without proving the path from the
pinned shortcut to the running installed product. The result was lost operator
time and a reasonable loss of trust.

## What the evidence says

The pinned shortcut is:

```text
C:\Users\m\AppData\Roaming\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk
```

Its verified target is:

```text
C:\Users\m\AppData\Local\Programs\Archetypes\launcher.exe
```

On 2026-09-09, the golden-path deployment was run again after the full Rust
test suite passed (80 engine, 16 launcher, and 5 Windows-identity tests). The
deployment verified these SHA-256 values after copying to the installed path:

| Binary | SHA-256 |
| --- | --- |
| `engine.exe` | `F6BC58C6B3449A74DFE703FC81086FDE4680BDB62957DF87F6257B82128C9E2B` |
| `launcher.exe` | `6947EDF95CC3226B3FABA78A6A623146A59EBF1EB92036D09CA41848F7A243CA` |

The same hashes were observed in `target\\release`, `dist`, and the installed
Programs tree. The deployment script also reread the persisted Taskbar `.lnk`
and verified both its target and working directory before returning success.

## The failure class

This was not a harmless wording problem. It was an invalid definition of done:
the agent equated a successful build or copy command with a successful buyer
deployment. That is a low acceptance bar. It leaves a gap between source truth
and the product the buyer actually launches.

The evidence does **not** establish a person's intent to deceive. It does
establish inaccurate completion claims and inadequate verification. Whether the
cause was haste, laziness, or an overly optimistic agent loop, the correction
is the same: claims must stop at the last independently witnessed fact.

## Concrete defects

1. The old deployment script could copy files and report completion without
   hashing the installed executables.
2. It did not reread the saved pinned shortcut to prove Explorer would launch
   the installed launcher.
3. It could attempt a copy while a buyer was still running the installed game,
   creating a misleading result or an unclear Windows file-lock error.
4. A source/build/test result was presented too close to a buyer-runtime
   result. Tests prove code behavior; they do not prove the pinned taskbar
   shortcut launched the intended image.
5. The manifestation screenshot remains evidence that the in-game waiting
   surface is not yet acceptable: it reports a generic timer but does not show
   live Chronos stages or a completed reveal. That capability must remain
   explicitly unfinished until a real prompt succeeds and the returned GLB is
   visibly mounted on the pedestal.

## Corrective controls now in source

`scripts/install_product.ps1` now fails before copying when `engine.exe` or
`launcher.exe` is live from the chosen installed root. It verifies post-copy
SHA-256 equality for both executables.

`scripts/install_shortcut.ps1` now rereads the Taskbar shortcut after saving it
and fails if its target or working directory is not the installed Programs
tree.

These controls make false-positive deployment claims harder. They do not by
themselves prove a foreground player action; that still requires launching from
the pinned icon and observing the process/window. That distinction is retained
on purpose.

## Acceptance standard going forward

A taskbar-launcher change is only deployable when all of the following are
recorded:

1. release build succeeds;
2. installed engine and launcher match staged hashes;
3. pinned `.lnk` target and working directory are reread and correct;
4. no stale installed process is silently left running; and
5. a foreground launch from the pinned icon is visually witnessed, with the
   launched process path identified.

Until item 5 exists, the honest status is **installed and path-verified, not
buyer-launch witnessed**.
