use avian3d::dynamics::solver::SolverConfig;
use avian3d::prelude::*;
use bevy::prelude::*;

// Module declarations - organized by domain
mod core;
mod gameplay;
mod physics;
mod rendering;
mod ui;

// Imports from modules
use core::{AppState, GameConfiguration, GameFonts, GameSetup, GameState, PlayerManager};
use gameplay::{
    ai_player_system, animate_match_victory, animate_victory, attach_throw_tracking,
    check_blocks_settled, check_out_of_bounds, check_round_complete, check_throw_settled,
    cleanup_out_of_bounds, detect_settled_camera_target, fade_in_blocks, fade_out_blocks,
    finalize_throw_performance, handle_input, handle_match_complete,
    remove_camera_target_after_settle, respawn_target_for_round, setup_arena, setup_lights,
    start_match_victory_animation, start_next_round, start_victory_animation, track_block_bounces,
    transition_to_fadeout, update_distances, AIThinkingTimer, ThrowHistory,
};
use rendering::{attach_lights_to_blocks, move_camera, setup_camera, update_block_lights};
use ui::{
    cleanup_menu_ui, handle_pause_input, handle_pause_menu, handle_player_selection,
    handle_start_menu, reset_game_state, setup_countdown, setup_pause_menu, setup_player_selection,
    setup_start_menu, setup_ui, show_turn_notification, update_button_colors, update_countdown,
    update_turn_notification, update_ui,
};

fn main() {
    // Load configuration from config.yaml
    let config = GameConfiguration::load_or_default();
    println!("=== BLOCCE - CONFIGURATION LOADED ===");
    println!(
        "Config: {} rounds to win, {} blocks per player",
        config.game.rounds_to_win, config.game.blocks_per_player
    );

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "blocce".to_string(),
            resolution: (1280, 720).into(),
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#blocce-canvas".into()),
            #[cfg(target_arch = "wasm32")]
            fit_canvas_to_parent: true,
            #[cfg(target_arch = "wasm32")]
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PhysicsPlugins::default().with_length_unit(1.0));

    // Initialize resources from config
    let game_state = GameState::from_config(&config);
    let player_manager = PlayerManager::from_config(&config);
    let clear_color = ClearColor(config.rendering.clear_color.to_srgb());
    let gravity = Vec3::new(0.0, config.physics.gravity, 0.0);

    app.insert_resource(clear_color)
        .insert_resource(Gravity(gravity))
        .insert_resource(SubstepCount(config.physics.substeps))
        .insert_resource(SolverConfig {
            max_overlap_solve_speed: config.physics.max_overlap_solve_speed,
            contact_frequency_factor: 1.5,
            restitution_iterations: config.physics.restitution_iterations as usize,
            ..default()
        })
        .insert_resource(config)
        .insert_resource(game_state)
        .insert_resource(player_manager)
        .insert_resource(GameSetup::default())
        .insert_resource(AIThinkingTimer::default())
        .insert_resource(ThrowHistory::default())
        // Add state management
        .init_state::<AppState>()
        // Spawn arena and camera on startup for menu background
        .add_systems(
            Startup,
            (setup_camera, setup_arena, setup_lights, load_fonts),
        )
        // Loading systems - transition to StartMenu once fonts are loaded
        .add_systems(
            Update,
            check_fonts_loaded.run_if(in_state(AppState::Loading)),
        )
        // Start Menu systems
        .add_systems(OnEnter(AppState::StartMenu), setup_start_menu)
        .add_systems(OnExit(AppState::StartMenu), cleanup_menu_ui)
        .add_systems(
            Update,
            (handle_start_menu, update_button_colors).run_if(in_state(AppState::StartMenu)),
        )
        // Player Selection systems
        .add_systems(OnEnter(AppState::PlayerSelection), setup_player_selection)
        .add_systems(OnExit(AppState::PlayerSelection), cleanup_menu_ui)
        .add_systems(
            Update,
            (handle_player_selection, update_button_colors)
                .run_if(in_state(AppState::PlayerSelection)),
        )
        // Countdown systems
        .add_systems(OnEnter(AppState::Countdown), setup_countdown)
        .add_systems(OnExit(AppState::Countdown), cleanup_menu_ui)
        .add_systems(
            Update,
            update_countdown.run_if(in_state(AppState::Countdown)),
        )
        // Game initialization when entering InGame - just UI since arena/camera already exist
        .add_systems(OnEnter(AppState::InGame), setup_ui)
        .add_systems(OnExit(AppState::InGame), cleanup_game_ui)
        // Game systems - only run during InGame state
        // Split into multiple groups due to tuple size limits
        .add_systems(
            Update,
            (
                handle_input,
                ai_player_system,
                attach_throw_tracking,
                track_block_bounces,
                finalize_throw_performance,
                move_camera,
                update_distances,
                check_out_of_bounds,
                cleanup_out_of_bounds,
                check_throw_settled,
                check_round_complete,
                check_blocks_settled,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                start_victory_animation,
                animate_victory,
                transition_to_fadeout,
                fade_in_blocks,
                fade_out_blocks,
                start_next_round,
                respawn_target_for_round,
                start_match_victory_animation,
                animate_match_victory,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                handle_match_complete,
                detect_settled_camera_target,
                remove_camera_target_after_settle,
                attach_lights_to_blocks,
                update_block_lights,
                update_ui,
                show_turn_notification,
                update_turn_notification,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Pause handling - runs during InGame state
        .add_systems(
            Update,
            handle_pause_input.run_if(in_state(AppState::InGame).or(in_state(AppState::PauseMenu))),
        )
        // Pause Menu systems
        .add_systems(OnEnter(AppState::PauseMenu), setup_pause_menu)
        .add_systems(OnExit(AppState::PauseMenu), cleanup_menu_ui)
        .add_systems(
            Update,
            (handle_pause_menu, update_button_colors).run_if(in_state(AppState::PauseMenu)),
        )
        // Cleanup when returning to menu
        .add_systems(
            OnEnter(AppState::StartMenu),
            (cleanup_game_entities, reset_game_state),
        )
        .run();
}

/// Load custom fonts at startup
fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameFonts::load(&asset_server));
}

/// Check if fonts are loaded and transition to StartMenu
fn check_fonts_loaded(
    fonts: Option<Res<GameFonts>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(fonts) = fonts {
        // Check if both fonts are loaded
        if asset_server.is_loaded_with_dependencies(&fonts.title)
            && asset_server.is_loaded_with_dependencies(&fonts.ui)
        {
            next_state.set(AppState::StartMenu);
        }
    }
}

/// Marker component for game entities that should be cleaned up
#[derive(Component)]
#[allow(dead_code)]
struct GameEntity;

/// Cleanup all game entities when returning to menu
/// Keep camera/arena/lights for background, only remove blocks and game UI
fn cleanup_game_entities(
    mut commands: Commands,
    blocks: Query<Entity, With<crate::core::Block>>,
    ui: Query<Entity, With<crate::ui::game_ui::GameUiRoot>>,
) {
    // Despawn blocks
    for entity in blocks.iter() {
        commands.entity(entity).despawn();
    }
    // Despawn game UI
    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }
    // Keep camera, arena, and lights for menu background
}

/// Remove the whole HUD before pausing or leaving the game.
fn cleanup_game_ui(
    mut commands: Commands,
    ui: Query<Entity, With<crate::ui::game_ui::GameUiRoot>>,
) {
    for entity in &ui {
        commands.entity(entity).despawn();
    }
}
