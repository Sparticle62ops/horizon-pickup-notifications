# horizon-pickup-notifications

Pickup Notifications (Roundaround/mc-fabric-pickup-notifications) ported to [Horizon](https://github.com/horizon-rust/horizon) as a native Rust mod.

Upstream: github.com/Roundaround/mc-fabric-pickup-notifications, commit aa29c0d, MIT (c) 2021 Evan Steinkerchner (Roundaround). LICENSE carried verbatim.

## Slice 1 (this repo)

1:1 logic core, no optimizations or behavior changes:

- Config: 11 trove options with upstream defaults, slider clamps, and the v1-to-v2 guiOffset migration
- IconAlignment and GuiAlignment values with screen-edge geometry and offset multipliers
- The item and experience pickup event buses
- S2C packet models (experience INT payload, item stack payload) and the server send gate
- The HUD queue state machine: shown list, backlog queue, merge-into-shown-extends, merge-into-queue, promote-when-room
- The notification base state machine: durations, pop, expiry, slide-in/out offset percent with partial ticks, right-aligned truth table
- Item notifications: same-item-same-components merging, count growth, unique-info display rules
- Experience notifications: vanilla orb-size thresholds and the sine-based icon color math

22 unit tests green.

## Known degradations

- Drawing (background fill, text, item icons, orb texture) is a seam: the port emits draw specs; the client applies them. Gated on the HUD-layer hook tracked in horizon-rust/horizon #6243.
- The server Touch hooks (ItemEntityMixin.playerTouch at awardStat, ExperienceOrbMixin.playerTouch at take) and the trove S2C transport are game-side seams.
