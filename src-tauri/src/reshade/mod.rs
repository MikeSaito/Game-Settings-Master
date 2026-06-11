mod api;
// ReShade addon: BSD 3-Clause — presets/reshade/LICENSE-ReShade.txt
// Shaders: presets/reshade/shaders/THIRD-PARTY-NOTICES.txt
mod bundle;
mod config;
mod detect;
mod game_presets;
mod gpu_adapt;
mod guard;
mod ini_edit;
mod install;
mod presets;
mod remove;
mod resolve;
mod vulkan_layer;

pub use api::{list_graphics_apis, GraphicsApi, GraphicsApiInfo};
pub use config::{
    effective_preset_for_game, load_settings, save_settings, should_prompt_api,
    ReShadePerGameSettings, ReShadeSettings,
};
pub use detect::{get_status, get_status_with_settings, ReShadeGameStatus};
pub use game_presets::{list_presets_for_game, suggested_reshade_api_for_game};
pub use gpu_adapt::adapt_preset_for_gpu;
pub use ini_edit::PresetOverrides;
pub use install::{
    apply_launch_reshade_policy, ensure_installed, install_reshade, update_preset,
    update_preset_parameters, InstallResult,
};
pub use presets::{list_presets, preset_details, PresetDetails, ReShadePresetInfo};
pub use remove::{remove_reshade, RemoveResult};
pub use resolve::resolve_install_target;
