use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Transform, TransformBundle},
    ecs::{Component, Entities, Entity, Join, Read, System,
          VecStorage, WriteStorage, ReadStorage, },
    input::{InputBundle, InputHandler},
    prelude::*,
    renderer::{
        ALPHA, Camera, ColorMask, DisplayConfig, DrawFlat2D, Flipped, Pipeline, PngFormat,
        Projection, RenderBundle, Sprite, SpriteRender, SpriteSheet,
        SpriteSheetHandle, Stage, Texture, TextureMetadata, SpriteSheetFormat, Transparent
    },
};
use specs_derive::Component;

const PLAYER_W: u32 = 90;
const TOTAL_PLAYER_SPRITE_HEIGHT: u32 = 184;
const PLAYER_SPRITE_Y_PADDING: u32 = 20; // pixels between sprites
const PLAYER_H: u32 = TOTAL_PLAYER_SPRITE_HEIGHT - PLAYER_SPRITE_Y_PADDING;
const GROUND_Y: f32 = 74.;
const CRATE_SIZE: f32 = 77.;
const DISPLAY_WIDTH: f32 = 1000.;

#[derive(PartialEq, Clone, Copy)]
pub enum PlayerState {
    Idle,
    Walking,
    Jumping,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::Idle
    }
}

pub struct TwoDimVector<T> {
    x: T,
    y: T,
}

impl Default for TwoDimVector<f32> {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct TwoDimObject {
    size: TwoDimVector<f32>,
    position: TwoDimVector<f32>,
    velocity: TwoDimVector<f32>,
}

impl TwoDimObject {
    fn new(width: f32, height: f32) -> Self {
        TwoDimObject {
            size: TwoDimVector { x: width, y: height },
            position: TwoDimVector { x: 0., y: 0. },
            velocity: TwoDimVector { x: 0., y: 0. },
        }
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.position = TwoDimVector { x, y };
    }

    fn set_velocity(&mut self, x: f32, y: f32) {
        self.velocity = TwoDimVector { x, y };
    }

    fn update_transform_position(&self, transform: &mut Transform) {
        transform.set_x(self.position.x);
        transform.set_y(self.position.y);
    }

    fn top(&self) -> f32 {
        self.position.y + self.size.y / 2.
    }

    fn set_top(&mut self, top: f32) {
        self.position.y = top - self.size.y / 2.;
    }

    fn bottom(&self) -> f32 {
        self.position.y - self.size.y / 2.
    }

    fn set_bottom(&mut self, bottom: f32) {
        self.position.y = bottom + self.size.y / 2.;
    }

    fn left(&self) -> f32 {
        self.position.x - self.size.x / 2.
    }

    fn set_left(&mut self, left: f32) {
        self.position.x = left + self.size.x / 2.;
    }

    fn right(&self) -> f32 {
        self.position.x + self.size.x / 2.
    }

    fn overlapping_x(&self, other: &Self) -> bool {
        self.right() >= other.left() && self.left() <= other.right()
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Player {
    ticks: usize,
    state: PlayerState,
    two_dim: TwoDimObject,
}

impl Player {
    fn new(two_dim: TwoDimObject) -> Self {
        Player {
            ticks: 0,
            state: PlayerState::Idle,
            two_dim,
        }
    }
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let background_sprite_sheet_handle =
            load_sprite_sheet(world, "./texture/BG.png", "./texture/BG.ron");
        init_background_sprite(world, &background_sprite_sheet_handle);

        let ground_sprite_sheet_handle =
            load_sprite_sheet(world, "./texture/ground.png", "./texture/ground.ron");
        init_ground_sprite(world, &ground_sprite_sheet_handle);

        let crate_sprite_sheet_handle =
            load_sprite_sheet(world, "./texture/Crate.png", "./texture/Crate.ron");
        init_crate_sprite(world, &crate_sprite_sheet_handle, 0., GROUND_Y);
        init_crate_sprite(world, &crate_sprite_sheet_handle, CRATE_SIZE, GROUND_Y);
        init_crate_sprite(world, &crate_sprite_sheet_handle, 0., GROUND_Y + CRATE_SIZE);

        world.register::<Player>();
        let sprite_sheet_handle = load_player_sprite_sheet(world);
        init_player(world, &sprite_sheet_handle);

        init_camera(world);
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
        .with_bundle(RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&[])
        )?
        .with(ControlSystem, "control_system", &[])
        .with(PhysicsSystem, "physics_system", &["control_system"])
        .with(PlayerAnimationSystem, "player_animation_system", &["physics_system"]);

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
            DISPLAY_WIDTH, // todo set this by screen size?
            0.0,
            1000.,
        )))
        .with(transform)
        .build();
}

fn init_player(world: &mut World, sprite_sheet_handle: &SpriteSheetHandle) -> Entity {
    let scale = 1.;

    let mut transform = Transform::default();
    transform.set_scale(scale, scale, scale);

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 60, // paddle is the first sprite in the sprite_sheet
    };

    let mut two_dim_object = TwoDimObject::new(PLAYER_W as f32, PLAYER_H as f32);
    two_dim_object.set_position(500., 500.);
    two_dim_object.update_transform_position(&mut transform);

    world
        .create_entity()
        .with(transform)
        .with(Player::new(two_dim_object))
        .with(sprite_render)
        .with(Transparent)
        .build()
}

fn init_background_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_xyz(500., 500., -10.);
    transform.set_scale(1., 1.5, 1.);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };
    world.create_entity()
        .with(transform)
        .with(sprite)
        .with(Transparent)
        .build()
}

fn init_ground_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_z(-9.);
    transform.set_scale(10., 1., 1.);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    let mut two_dim_object = TwoDimObject::new(1280., 128.);
    two_dim_object.set_left(0.);
    two_dim_object.set_top(GROUND_Y);
    two_dim_object.update_transform_position(&mut transform);

    world.create_entity()
        .with(transform)
        .with(two_dim_object)
        .with(sprite)
        .with(Transparent)
        .build()
}

fn init_crate_sprite(world: &mut World, sprite_sheet: &SpriteSheetHandle, left: f32, bottom: f32) -> Entity {
    let mut transform = Transform::default();
    transform.set_z(-9.);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    let mut two_dim_object = TwoDimObject::new(CRATE_SIZE, CRATE_SIZE);
    two_dim_object.set_left(left);
    two_dim_object.set_bottom(bottom);
    two_dim_object.update_transform_position(&mut transform);

    world.create_entity()
        .with(transform)
        .with(two_dim_object)
        .with(sprite)
        .with(Transparent)
        .build()
}

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat,
        texture_handle,
        (),
        &sprite_sheet_store,
    )
}

fn load_player_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
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

    for i in 0..(sprite_count as u32) {
        let offset_x = 0;
        let offset_y = TOTAL_PLAYER_SPRITE_HEIGHT * i;
        let offsets = [0.; 2]; // Align the sprite with the middle of the entity.

        let sprite = Sprite::from_pixel_values(
            image_w, image_h, PLAYER_W, PLAYER_H, offset_x, offset_y, offsets,
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

            player.two_dim.velocity.x = x_input as f32;

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
            player.two_dim.position.x += player.two_dim.velocity.x;

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
                }
            }

            player.two_dim.set_bottom(new_y);

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
                    else if player.two_dim.velocity.x != 0. { PlayerState::Walking }
                    else { PlayerState::Idle };

            if current_state != next_state {
                player.state = next_state;
                player.ticks = 0; // reset animation if player state changed
            }

            let (sprite_initial_index, num_sprites) = match player.state {
                PlayerState::Idle => (15, 15),
                PlayerState::Walking => (60, 15),
                PlayerState::Jumping => (35, 7),
            };
            let game_frames_per_animation_frame = 6;
            sprite.sprite_number = (player.ticks / game_frames_per_animation_frame) % num_sprites + sprite_initial_index;
            player.ticks = player.ticks.wrapping_add(1);

            player.two_dim.update_transform_position(&mut transform);
        }
    }
}