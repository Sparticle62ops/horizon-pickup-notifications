//! Exact 1:1 translation of Pickup Notifications (upstream Java, MIT,
//! (c) 2021 Evan Steinkerchner (Roundaround)), commit aa29c0d. The Minecraft
//! and Fabric APIs that do not exist on Horizon are host seams with the same
//! shapes and semantics.
//!
//! The mod API wires what the pinned SDK (api 0.2, f00125f61f) reaches:
//! - the original's config options the Settings page can hold, same
//!   defaults. Ten of the eleven register as settings; the `guiOffset`
//!   Position has no paired-offset kind on the page, so it stays a
//!   config-file value. Upstream's float sliders carry a step COUNT (trove
//!   slider semantics), translated to step sizes: guiScale 30 steps over
//!   0.1..=3, backgroundOpacity 20 steps over 0..=1. `maxNotifications` is
//!   an unbounded int upstream (at least 1), so its slider caps at 10 while
//!   the config file keeps the unclamped value;
//! - `client_setup` runs the mod entry 1:1: load
//!   `config/pickupnotifications/config.json` (the v1-to-v2 migration is
//!   the core's `perform_config_update`), then sync the settings into the
//!   live config cell the HUD queue reads.
//!
//! Gaps, tracked: the item/experience pickup events and the S2C transport
//! are game-side seams (upstream's ItemEntity/ExperienceOrb touch hooks and
//! TroveNetworking); the HUD draw waits on the HUD-layer hook tracked in
//! horizon-rust/horizon #6243, and until it lands the port keeps emitting
//! the same draw specs the client would apply. The whole config model,
//! event buses, packet models and HUD state machine carry 1:1 in `core`,
//! 22 tests green.

use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use mod_api::{ClientSetupEvent, ClientHost, RegistrationContext, Registrar, SettingSpec};

pub mod core;

/// The game's config folder (upstream: FabricLoader's config dir).
pub fn game_config_dir() -> PathBuf {
    std::env::var_os("PICKUP_NOTIFICATIONS_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config"))
}

/// `config/pickupnotifications/config.json`.
pub fn config_path() -> PathBuf {
    game_config_dir().join(core::MOD_ID).join("config.json")
}

/// The live config the HUD queue and draw specs read (upstream: the mod's
/// config instance fields).
static LIVE: OnceLock<RwLock<core::Config>> = OnceLock::new();

pub fn set_live_config(config: core::Config) {
    let cell = LIVE.get_or_init(|| RwLock::new(core::Config::default()));
    *cell.write().expect("the config cell is not poisoned") = config;
}

pub fn live_config() -> core::Config {
    LIVE.get()
        .and_then(|cell| cell.read().ok())
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// The original's config options as settings, same defaults.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        let _ = registrar.register_setting(SettingSpec::toggle(
            "mod_enabled",
            "setting.pickupnotifications.modEnabled",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "track_experience",
            "setting.pickupnotifications.trackExperience",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "gui_alignment",
            "setting.pickupnotifications.guiAlignment",
            [
                "option.pickupnotifications.gui_alignment.top_left",
                "option.pickupnotifications.gui_alignment.top_right",
                "option.pickupnotifications.gui_alignment.bottom_left",
                "option.pickupnotifications.gui_alignment.bottom_right",
            ],
            0,
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            "gui_scale",
            "setting.pickupnotifications.guiScale",
            0.1,
            3.0,
            2.9 / 30.0,
            1.0,
        ));
        // Upstream is an unbounded int (at least 1); the slider caps at 10
        // and the config file keeps the unclamped value.
        let _ = registrar.register_setting(SettingSpec::range(
            "max_notifications",
            "setting.pickupnotifications.maxNotifications",
            1.0,
            10.0,
            1.0,
            4.0,
        ));
        let _ = registrar.register_setting(SettingSpec::choice(
            "icon_alignment",
            "setting.pickupnotifications.iconAlignment",
            [
                "option.pickupnotifications.icon_alignment.outside",
                "option.pickupnotifications.icon_alignment.inside",
                "option.pickupnotifications.icon_alignment.left",
                "option.pickupnotifications.icon_alignment.right",
            ],
            0,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "show_unique_info",
            "setting.pickupnotifications.showUniqueInfo",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "render_background",
            "setting.pickupnotifications.renderBackground",
            true,
        ));
        let _ = registrar.register_setting(SettingSpec::range(
            "background_opacity",
            "setting.pickupnotifications.backgroundOpacity",
            0.0,
            1.0,
            0.05,
            0.5,
        ));
        let _ = registrar.register_setting(SettingSpec::toggle(
            "render_shadow",
            "setting.pickupnotifications.renderShadow",
            false,
        ));
    }

    /// 1:1 of the original's mod init: load the config file (defaults when
    /// missing or broken), then sync the settings into the live config cell
    /// the HUD queue reads.
    fn client_setup(&self, host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        let mut config = std::fs::read_to_string(config_path())
            .ok()
            .and_then(|text| core::Config::from_json(&text).ok())
            .unwrap_or_default();
        if let Some(value) = host.setting_bool("mod_enabled") {
            config.mod_enabled = value;
        }
        if let Some(value) = host.setting_bool("track_experience") {
            config.track_experience = value;
        }
        if let Some(index) = host.setting_choice("gui_alignment") {
            let index = (index as usize).min(core::GuiAlignment::ALL.len() - 1);
            config.gui_alignment = core::GuiAlignment::ALL[index];
        }
        if let Some(value) = host.setting_f64("gui_scale") {
            config.gui_scale = value as f32;
        }
        if let Some(value) = host.setting_f64("max_notifications") {
            config.max_notifications = value as i32;
        }
        if let Some(index) = host.setting_choice("icon_alignment") {
            let index = (index as usize).min(core::IconAlignment::ALL.len() - 1);
            config.icon_alignment = core::IconAlignment::ALL[index];
        }
        if let Some(value) = host.setting_bool("show_unique_info") {
            config.show_unique_info = value;
        }
        if let Some(value) = host.setting_bool("render_background") {
            config.render_background = value;
        }
        if let Some(value) = host.setting_f64("background_opacity") {
            config.background_opacity = value as f32;
        }
        if let Some(value) = host.setting_bool("render_shadow") {
            config.render_shadow = value;
        }
        config.ensure_valid();
        set_live_config(config);
    }
}
