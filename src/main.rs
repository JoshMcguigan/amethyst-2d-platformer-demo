use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Transform, TransformBundle},
    ecs::{Entity, Read, ReadStorage, WriteStorage, System, Join, Component, DenseVecStorage,
          NullStorage, Entities, },
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawFlat2D, Pipeline, PngFormat, Projection, RenderBundle, Stage,
        Texture, TextureHandle, TextureMetadata, ALPHA, ColorMask, ScreenDimensions, Flipped,
        SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, SpriteRender,
    },
    input::{InputBundle, InputHandler},
};
use specs_derive::{Component};

#[derive(Default, Component)]
#[storage(NullStorage)]
pub struct Player;

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
    let width = 614;
    let height = 564;
    let scale = 0.3;

    let mut transform = Transform::default();
    transform.set_x(500.);
    transform.set_y((height as f32 * scale) / 2.);

    transform.set_scale(scale, scale, scale);

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0, // paddle is the first sprite in the sprite_sheet
    };


    world
        .create_entity()
        .with(transform)
        .with(Player)
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
            "./texture/css_sprites.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "./texture/css_sprites.ron", // Here we load the associated ron file
        SpriteSheetFormat,
        texture_handle, // We pass it the texture we want it to use
        (),
        &sprite_sheet_store,
    )
}

pub struct ControlSystem;

impl<'s> System<'s> for ControlSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<String, String>>,
        WriteStorage<'s, Flipped>,
    );

    fn run(&mut self, (entity, mut transforms, player, input, mut flipped): Self::SystemData) {
        for (mut transform, player, e) in (&mut transforms, &player, &*entity).join() {
            if let Some(mv_amount) = input.axis_value("horizontal") {
                let player_x = transform.translation().x;
                transform.set_x(
                    player_x + mv_amount as f32
                );
                if mv_amount > 0. {
                    // face right
                    flipped.remove(e);
                } else if mv_amount < 0. {
                    // face left
                    flipped.insert(e, Flipped::Horizontal);
                }

            }
        }
    }
}

pub struct PlayerAnimationSystem;

impl<'s> System<'s> for PlayerAnimationSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, SpriteRender>,
    );

    fn run(&mut self, (players, mut sprites): Self::SystemData) {
        for (mut sprite) in (&mut sprites).join() {
            if sprite.sprite_number == 1 {
                sprite.sprite_number = 0;
            } else {
                sprite.sprite_number = 1;
            }
        }
    }
}