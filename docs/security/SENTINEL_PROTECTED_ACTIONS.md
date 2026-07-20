# Sentinel Protected Actions

Product: `Archetypes`
Release Handling: every canonical protected action is classified here. `blocked` means the action remains a release blocker until executable proof exists.

| Protected Action | Release Handling |
| --- | --- |
| `agent.spawn` | not used directly; must deny unless introduced and certified. |
| `artifact.register` | blocked; game artifact sealing not fully certified. |
| `artifact.export` | blocked; export/share paths require Sentinel mediation. |
| `artifact.use` | blocked; artifact-use path not fully certified. |
| `browser.navigate_external` | blocked if external links are enabled. |
| `capability.issue` | blocked; capability lifecycle not fully wired in game runtime. |
| `capability.consume` | blocked; capability consumption not fully wired in game runtime. |
| `chat.respond` | blocked; dialogue response path requires runtime Sentinel mediation. |
| `effect.execute` | partial; launcher launch intent is guarded, full game effect coverage blocked. |
| `external_message.send` | blocked; sharing/outbound comms require Sentinel mediation. |
| `file.delete` | blocked; local deletion paths require mediation. |
| `file.read_sensitive` | blocked; profile/save reads require mediation. |
| `file.write` | blocked; save/profile writes require mediation. |
| `game.respond` | blocked; in-engine game response mediation incomplete. |
| `game.share` | blocked; share/export mediation incomplete. |
| `hardware.activate_camera` | not used directly; must deny unless introduced and certified. |
| `hardware.activate_microphone` | blocked if voice input is enabled. |
| `identity.genesis` | blocked; game identity lifecycle not release-certified. |
| `identity.register` | blocked; player identity lifecycle not release-certified. |
| `identity.rebind` | blocked; player identity lifecycle not release-certified. |
| `identity.key.register` | blocked; key lifecycle not release-certified. |
| `identity.key.revoke` | blocked; key lifecycle not release-certified. |
| `identity.key.rotate` | blocked; key lifecycle not release-certified. |
| `installer.update` | blocked; installer/update certification incomplete. |
| `memory.write` | blocked; in-engine memory writes require mediation. |
| `memory.delete` | blocked; in-engine memory deletion requires mediation. |
| `model.generate` | blocked; model generation mediation incomplete. |
| `network.egress` | blocked; outbound network mediation incomplete. |
| `network.request` | blocked; network request mediation incomplete. |
| `payment.or_commitment` | not used directly; must deny unless introduced and certified. |
| `plugin.install` | blocked if mods/plugins are introduced. |
| `plugin.execute` | blocked if mods/plugins are introduced. |
| `policy.evaluate` | partial; launcher relies on Chronos policy path, in-engine policy lifecycle blocked. |
| `process.spawn` | partial; launcher engine spawn is Sentinel-gated, child runtime coverage blocked. |
| `profile.generate` | blocked; player profile generation mediation incomplete. |
| `robot.command` | not used directly; must deny unless introduced and certified. |
| `shell.execute` | not used directly; must deny unless introduced and certified. |
| `system.install` | blocked; installer/system paths require certification. |
| `tool.invoke` | blocked; tool invocation mediation incomplete. |
| `tool.run` | blocked; tool execution mediation incomplete. |

