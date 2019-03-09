use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Transform, TransformBundle},
    ecs::{Component, Entities, Entity, Join, Read, System,
          VecStorage, WriteStorage, },
    input::{InputBundle, InputHandler},
    prelude::*,
    renderer::{
        ALPHA, Camera, ColorMask, DisplayConfig, DrawFlat2D, Flipped, Pipeline, PngFormat,
        Projection, RenderBundle, Sprite, SpriteRender, SpriteSheet,
        SpriteSheetHandle, Stage, Texture, TextureMetadata
    },
};
use specs_derive::Component;

#[derive(PartialEq, Clone, Copy)]
pub enum PlayerState {
    Idle,
    Walking,
    JumpingPosVelocity,
    JumpingNegVelocity,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Idle
    }
}

#[derive(Default, Component)]
#[storage(VecStorage)]
pub struct Player {
    ticks: usize,
    state: PlayerState,
    y_velocity: f32,
}

impl Player {
    fn new() -> Self {
        Player {
            ticks: 0,
            state: PlayerState::Idle,
            y_velocity: 0.,
        }
    }
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let sprite_sheet_handle = load_sprite_sheet(world);

        world.register::<Player>();
        init_player(world, &sprite_sheet_handle);

        init_camera(world)
    }
}

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
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with(ControlSystem, "control_system", &[])
        .with(PlayerAnimationSystem, "player_animation_system", &[]);

    let mut game = Application::build("./", Example)?.build(game_data)?;
    game.run();

    Ok(())
}

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_xyz(0.0, 0.0, 1.0);

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            1000., // todo set this by screen size?
            0.0,
            1000.,
        )))
        .with(transform)
        .build();
}

fn init_player(world: &mut World, sprite_sheet_handle: &SpriteSheetHandle) -> Entity {
    let _width = 200;
    let height = 164;
    let scale = 1.;


    let mut transform = Transform::default();
    transform.set_x(500.);
    transform.set_y((height as f32 * scale) / 2.);

    transform.set_scale(scale, scale, scale);

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 60, // paddle is the first sprite in the sprite_sheet
    };


    world
        .create_entity()
        .with(transform)
        .with(Player::new())
        .with(sprite_render)
        .build()
}

fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `sprite_sheet` is the layout of the sprites on the image
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "./texture/spritesheet.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();

    let sprite_count = 75; // number of sprites
    let mut sprites = Vec::with_capacity(sprite_count);

    let image_w = 200;
    let image_h = 13980;
    let sprite_w = 90;
    let total_sprite_height = 184;
    let sprite_y_padding = 20; // pixels between sprites
    let sprite_h = total_sprite_height - sprite_y_padding;

    for i in 0..(sprite_count as u32) {
        let offset_x = 0;
        let offset_y = total_sprite_height * i;
        let offsets = [0.; 2]; // Align the sprite with the middle of the entity.

        let sprite = Sprite::from_pixel_values(
            image_w, image_h, sprite_w, sprite_h, offset_x, offset_y, offsets,
        );
        sprites.push(sprite);
    }

    let sprite_sheet = SpriteSheet {
        texture: texture_handle,
        sprites,
    };

    loader.load_from_data(
        sprite_sheet,
        (),
        &world.read_resource::<AssetStorage<SpriteSheet>>(),
    )
}

pub struct ControlSystem;

impl<'s> System<'s> for ControlSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        Read<'s, InputHandler<String, String>>,
        WriteStorage<'s, Flipped>,
    );

    fn run(&mut self, (entity, mut transforms, mut players, input, mut flipped): Self::SystemData) {
        for (transform, e, mut player) in (&mut transforms, &*entity, &mut players).join() {
            let current_state = player.state;
            let mut next_state = PlayerState::Idle; // assume idle until movement detected

            let mv_amount = input.axis_value("horizontal").expect("horizontal axis exists");
            let player_x = transform.translation().x;
            transform.set_x(
                player_x + mv_amount as f32
            );
            if mv_amount > 0. {
                // face right
                flipped.remove(e);
            } else if mv_amount < 0. {
                // face left
                flipped.insert(e, Flipped::Horizontal)
                    .expect("Failed to flip");
            }

            if mv_amount != 0. {
                next_state = PlayerState::Walking;
            }

            let player_y = transform.translation().y;
            let ground_level = 164. / 2.;
            let new_y = (player_y + player.y_velocity).max(ground_level); // todo this should consider platforms
            transform.set_y(new_y);

            let player_on_ground = new_y == ground_level; // todo this should consider platforms

            if player_on_ground {
                if input.action_is_down("jump")
                    .expect("jump action exists") {
                    player.y_velocity = 20.;
                    next_state = PlayerState::JumpingPosVelocity;
                } else {
                    player.y_velocity = 0.;
                };
            } else {
                if player.y_velocity > 0. {
                    next_state = PlayerState::JumpingPosVelocity;
                } else {
                    next_state = PlayerState::JumpingNegVelocity;
                }
                // gravity
                player.y_velocity -= 0.7;
            }

            if current_state != next_state {
                player.state = next_state;
                player.ticks = 0;
            }
        }
    }
}

pub struct PlayerAnimationSystem;

impl<'s> System<'s> for PlayerAnimationSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, SpriteRender>,
    );

    fn run(&mut self, (mut players, mut sprites): Self::SystemData) {
        for (mut player, mut sprite) in (&mut players, &mut sprites).join() {
            player.ticks = player.ticks.wrapping_add(1);
            let (sprite_initial_index, num_sprites) = match player.state {
                PlayerState::Idle => (15, 15),
                PlayerState::Walking => (60, 15),
                PlayerState::JumpingPosVelocity => (35, 7),
                PlayerState::JumpingNegVelocity => (35, 7),
            };
            let game_frames_per_animation_frame = 6;
            sprite.sprite_number = (player.ticks / game_frames_per_animation_frame) % num_sprites + sprite_initial_index;
        }
    }
}