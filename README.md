# horizon-pickup-notifications

Pickup Notifications (Roundaround/mc-fabric-pickup-notifications) ported to [Horizon](https://github.com/horizon-rust/horizon) as a native Rust mod.

Upstream: github.com/Roundaround/mc-fabric-pickup-notifications, commit aa29c0d, MIT (c) 2021 Evan Steinkerchner (Roundaround). LICENSE carried verbatim.

## What the mod API wires (this repo)

1:1 logic core, no optimizations or behavior changes, wired into the mod API:

- Settings: ten of the eleven config options register on the Settings page with upstream defaults and lang labels and tooltips (the guiOffset Position stays a config-file value; upstream's float sliders carry a step count, translated to step sizes: guiScale 30 steps over 0.1 to 3, backgroundOpacity 20 steps over 0 to 1; maxNotifications is an unbounded int upstream, so its slider caps at 10 while the config file keeps the unclamped value)
- client_setup: loads config/pickupnotifications/config.json (defaults when missing or broken; the v1-to-v2 guiOffset migration included), then syncs the settings into the live config cell the HUD queue reads
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
