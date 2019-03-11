use amethyst::{
    core::{Transform, TransformBundle},
    ecs::{Entities, Join, Read, ReadStorage,
          System, VecStorage, WriteStorage, },
    input::{InputBundle, InputHandler},
    prelude::*,
    renderer::{
        ALPHA, ColorMask, DisplayConfig, DrawFlat2D, Flipped, Pipeline,
        RenderBundle, SpriteRender,
        Stage
    },
};

mod entities;
use entities::{InitialState};
mod components;
use components::{Player, TwoDimObject, PlayerState};

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
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None)),
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
        .with(PlayerAnimationSystem, "player_animation_system", &["physics_system"]);

    let mut game = Application::build("./", InitialState)?.build(game_data)?;
    game.run();

    Ok(())
}

pub struct ControlSystem;

impl<'s> System<'s> for ControlSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, TwoDimObject>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (entities, mut players, two_dim_objects, input): Self::SystemData) {
        // calculate this so we know if the character should be able to jump
        let mut player_entities_on_ground = vec![];

        for (player, player_entity) in (&players, &entities).join() {
            for two_dim_object in (&two_dim_objects).join() {
                if player.two_dim.bottom() == two_dim_object.top() {
                    player_entities_on_ground.push(player_entity);
                }
            }
        }

        for (mut player, player_entity) in (&mut players, &entities).join() {
            let player_on_ground = player_entities_on_ground.contains(&player_entity);

            let x_input = input.axis_value("horizontal").expect("horizontal axis exists");
            let jump_input = input.action_is_down("jump").expect("jump action exists");

            if x_input == 0. {
                player.two_dim.velocity.x = 0.;
            } else {
                player.two_dim.velocity.x += 0.1 * x_input as f32;
                player.two_dim.velocity.x = player.two_dim.velocity.x.min(PLAYER_MAX_X_VELOCITY).max(-1. * PLAYER_MAX_X_VELOCITY);
            }

            if jump_input && player_on_ground {
                player.two_dim.velocity.y = 20.;
            };
        }
    }
}

pub struct PhysicsSystem;

impl<'s> System<'s> for PhysicsSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, TwoDimObject>,
    );

    fn run(&mut self, (mut players, two_dim_objects): Self::SystemData) {
        for mut player in (&mut players).join() {
            if player.two_dim.velocity.x > 0. {
                // player moving right
                let old_x = player.two_dim.right();
                let mut possible_new_x = old_x + player.two_dim.velocity.x;

                for two_dim_object in (&two_dim_objects).join() {
                    if player.two_dim.overlapping_y(two_dim_object)
                        && old_x <= two_dim_object.left()
                        && possible_new_x >= two_dim_object.left() {
                        // can't early return here, because we need to consider collision with more than one other object
                        // don't need to set velocity back to zero here, but could depending on how we want the player animation to act
                        possible_new_x = two_dim_object.left();
                    }
                }
                // ensure player stays inside "walls" of display
                let new_x = possible_new_x.min(DISPLAY_WIDTH).max(PLAYER_W as f32);
                player.two_dim.set_right(new_x);
            } else if player.two_dim.velocity.x < 0. {
                // player moving left
                let old_x = player.two_dim.left();
                let mut possible_new_x = old_x + player.two_dim.velocity.x;

                for two_dim_object in (&two_dim_objects).join() {
                    if player.two_dim.overlapping_y(two_dim_object)
                        && old_x >= two_dim_object.right()
                        && possible_new_x <= two_dim_object.right() {
                        // can't early return here, because we need to consider collision with more than one other object
                        // don't need to set velocity back to zero here, but could depending on how we want the player animation to act
                        possible_new_x = two_dim_object.right();
                    }
                }
                // ensure player stays inside "walls" of display
                let new_x = possible_new_x.min(DISPLAY_WIDTH - PLAYER_W as f32).max(0.);
                player.two_dim.set_left(new_x);
            };

            let player_on_ground = if player.two_dim.velocity.y > 0. {
                let old_y = player.two_dim.top();
                let possible_new_y = player.two_dim.top() + player.two_dim.velocity.y;
                let mut new_y = possible_new_y;

                for two_dim_object in (&two_dim_objects).join() {
                    if player.two_dim.overlapping_x(two_dim_object)
                        && old_y <= two_dim_object.bottom()
                        && new_y >= two_dim_object.bottom() {
                        new_y = two_dim_object.bottom();
                        player.two_dim.velocity.y = 0.;
                    }
                }
                player.two_dim.set_top(new_y);

                false
            } else if player.two_dim.velocity.y < 0. {
                let old_y = player.two_dim.bottom();
                let possible_new_y = player.two_dim.bottom() + player.two_dim.velocity.y;
                let mut new_y = possible_new_y;
                let mut player_on_ground = false;

                for two_dim_object in (&two_dim_objects).join() {
                    if player.two_dim.overlapping_x(two_dim_object)
                        && old_y >= two_dim_object.top()
                        && new_y <= two_dim_object.top() {
                        player_on_ground = true;
                        new_y = two_dim_object.top();
                        player.two_dim.velocity.y = 0.;
                    }
                }
                player.two_dim.set_bottom(new_y);

                player_on_ground
            } else {
                let mut player_on_ground = false;

                for two_dim_object in (&two_dim_objects).join() {
                    if player.two_dim.overlapping_x(two_dim_object)
                        && player.two_dim.bottom() == two_dim_object.top() {
                        player_on_ground = true;
                    }
                }

                player_on_ground
            };

            // gravity
            if player_on_ground {
                player.two_dim.velocity.y = 0.;
            } else {
                player.two_dim.velocity.y -= 0.7;
            }
        }
    }
}

pub struct PlayerAnimationSystem;

impl<'s> System<'s> for PlayerAnimationSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Flipped>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (entities, mut players, mut sprites, mut flipped, mut transforms): Self::SystemData) {
        for (player_entity, mut player, mut sprite, mut transform) in (&entities, &mut players, &mut sprites, &mut transforms).join() {
            // set sprite direction
            if player.two_dim.velocity.x > 0. {
                // face right
                flipped.remove(player_entity);
            } else if player.two_dim.velocity.x < 0. {
                // face left
                flipped.insert(player_entity, Flipped::Horizontal)
                    .expect("Failed to flip");
            }

            // set player state
            let current_state = player.state;
            let next_state =
                if player.two_dim.velocity.y != 0. { PlayerState::Jumping }
                    else if player.two_dim.velocity.x.abs() > PLAYER_MAX_X_VELOCITY * 0.7 { PlayerState::Running }
                    else if player.two_dim.velocity.x != 0. { PlayerState::Walking }
                    else { PlayerState::Idle };

            if current_state != next_state {
                player.state = next_state;
                player.ticks = 0; // reset animation if player state changed
            }

            let (sprite_initial_index, num_sprites) = match player.state {
                PlayerState::Idle => (15, 15),
                PlayerState::Walking => (60, 15),
                PlayerState::Running => (45, 15),
                PlayerState::Jumping => (35, 7),
            };
            let game_frames_per_animation_frame = 6;
            sprite.sprite_number = (player.ticks / game_frames_per_animation_frame) % num_sprites + sprite_initial_index;
            player.ticks = player.ticks.wrapping_add(1);

            player.two_dim.update_transform_position(&mut transform);
        }
    }
}