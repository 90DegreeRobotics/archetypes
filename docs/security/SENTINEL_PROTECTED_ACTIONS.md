# Sentinel Protected Actions

Product: `Archetypes`
Release Handling: every canonical protected action is classified here. Mediated actions have executable engine or launcher proof. Unused actions fail closed.

| Protected Action | Release Handling |
| --- | --- |
| `agent.spawn` | not used directly; fail-closed deny unless introduced and certified. |
| `artifact.register` | mediated; Chronos artifact requests authorize before Director pipeline. |
| `artifact.export` | not used directly; fail-closed deny. |
| `artifact.use` | not used directly; fail-closed deny. |
| `browser.navigate_external` | not used directly; fail-closed deny. |
| `capability.issue` | not used directly; fail-closed deny. |
| `capability.consume` | not used directly; fail-closed deny. |
| `chat.respond` | mediated; Ollama council and Consciousness replies authorize first. |
| `effect.execute` | partial; launcher launch intent is guarded with a client-signed body-bound Sentinel envelope. |
| `external_message.send` | not used directly; fail-closed deny. |
| `file.delete` | not used directly; fail-closed deny. |
| `file.read_sensitive` | not used directly; fail-closed deny. |
| `file.write` | mediated for world-memory and profile persist via `memory.write` / `profile.generate`. |
| `game.respond` | mediated; council and Consciousness replies travel the chat.respond gate. |
| `game.share` | not used directly; fail-closed deny. |
| `hardware.activate_camera` | not used directly; fail-closed deny. |
| `hardware.activate_microphone` | not used directly; fail-closed deny. |
| `identity.genesis` | not used directly; fail-closed deny. |
| `identity.register` | not used directly; fail-closed deny. |
| `identity.rebind` | not used directly; fail-closed deny. |
| `identity.key.register` | partial; launcher/engine client-key registration is wired through Chronos local authority bootstrap. |
| `identity.key.revoke` | not used directly; fail-closed deny. |
| `identity.key.rotate` | not used directly; fail-closed deny. |
| `installer.update` | not used directly; fail-closed deny. |
| `memory.write` | mediated; ledger append and world-memory lineage authorize first. |
| `memory.delete` | not used directly; fail-closed deny. |
| `model.generate` | mediated; Chronos artifact generation authorizes first. |
| `network.egress` | not used directly; fail-closed deny. |
| `network.request` | not used directly; fail-closed deny. |
| `payment.or_commitment` | not used directly; fail-closed deny. |
| `plugin.install` | not used directly; fail-closed deny. |
| `plugin.execute` | not used directly; fail-closed deny. |
| `policy.evaluate` | partial; launcher and engine rely on Chronos policy path with signed request binding. |
| `process.spawn` | partial; launcher engine spawn is Sentinel-gated by a signed launch append. Sidecar starts are local service supervision, not player-facing process.spawn. |
| `profile.generate` | mediated; Witness profile seal authorizes first. |
| `robot.command` | not used directly; fail-closed deny. |
| `shell.execute` | not used directly; fail-closed deny. |
| `system.install` | not used directly; fail-closed deny. |
| `tool.invoke` | not used directly; fail-closed deny. |
| `tool.run` | not used directly; fail-closed deny. |
