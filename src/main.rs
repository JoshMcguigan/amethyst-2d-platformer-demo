use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Transform, TransformBundle},
    ecs::{Entity, Read, ReadStorage, WriteStorage, System, Join, Component, DenseVecStorage},
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawFlat2D, Pipeline, PngFormat, Projection, RenderBundle, Stage,
        Texture, TextureHandle, TextureMetadata, ALPHA, ColorMask, ScreenDimensions,
    },
    input::{InputBundle, InputHandler}
};

pub struct Player;

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let texture_handle = load_texture(world, "./texture/boy/Idle (1).png");

        world.register::<Player>();

        let _image = init_image(world, &texture_handle);

        init_camera(world)
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let config = DisplayConfig::load("./resources/display_config.ron");
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None)),
    );
    let input_bundle = InputBundle::<String, String>::new()
        .with_bindings_from_file("./resources/bindings_config.ron")?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with(ControlSystem, "control_system", &[]);

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

fn init_image(world: &mut World, texture: &TextureHandle) -> Entity {
    let width = 614;
    let height = 564;
    let scale = 0.3;

    let mut transform = Transform::default();
    transform.set_x(500.);
    transform.set_y((height as f32 * scale) / 2.);

    transform.set_scale(scale, scale, scale);

    world
        .create_entity()
        .with(transform)
        .with(Player)
        .with(texture.clone())
        .build()
}

fn load_texture(world: &mut World, png_path: &str) -> TextureHandle {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        png_path,
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &texture_storage,
    )
}


pub struct ControlSystem;

impl<'s> System<'s> for ControlSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (mut transforms, player, input): Self::SystemData) {
        for (mut transform, _) in (&mut transforms, &player).join() {
            if let Some(mv_amount) = input.axis_value("horizontal") {
                let player_x = transform.translation().x;
                transform.set_x(
                    player_x + mv_amount as f32
                );
            }
        }
    }
}