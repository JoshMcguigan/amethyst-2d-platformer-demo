use amethyst::{
    core::{TransformBundle},
    input::{InputBundle},
    prelude::*,
    renderer::{ALPHA, ColorMask, DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage},
};

mod entities;
use entities::{InitialState};
mod components;
mod systems;
use systems::{ControlSystem, PhysicsSystem, AnimationSystem};

pub const PLAYER_W: u32 = 90;
pub const TOTAL_PLAYER_SPRITE_HEIGHT: u32 = 184;
pub const PLAYER_SPRITE_Y_PADDING: u32 = 20; // pixels between sprites
pub const PLAYER_H: u32 = TOTAL_PLAYER_SPRITE_HEIGHT - PLAYER_SPRITE_Y_PADDING;
pub const GROUND_Y: f32 = 74.;
pub const CRATE_SIZE: f32 = 77.;
pub const DISPLAY_WIDTH: f32 = 1000.;
pub const PLAYER_MAX_X_VELOCITY: f32 = 5.;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let config = DisplayConfig::load("./resources/display_config.ron");
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.2, 1.0], 1.0)
            .with_pass(
                DrawFlat2D::new()
                    .with_transparency(ColorMask::all(), ALPHA, None)
            ),
    );
    let input_bundle = InputBundle::<String, String>::new()
        .with_bindings_from_file("./resources/bindings_config.ron")?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&[])
        )?
        .with(ControlSystem, "control_system", &[])
        .with(PhysicsSystem, "physics_system", &["control_system"])
        .with(AnimationSystem, "animation_system", &["physics_system"]);

    let mut game =
        Application::build("./", InitialState)?.build(game_data)?;
    game.run();

    Ok(())
}