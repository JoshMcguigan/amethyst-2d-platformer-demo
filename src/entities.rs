use amethyst::{
    assets::{AssetStorage, Loader},
    core::{Transform},
    ecs::{Entity, Join, Read, System,
          VecStorage, WriteStorage, ReadStorage, },
    input::{InputBundle, InputHandler},
    prelude::*,
    renderer::{
        Camera, Pipeline, PngFormat,
        Projection, RenderBundle, Sprite, SpriteRender, SpriteSheet,
        SpriteSheetHandle, Stage, Texture, TextureMetadata, SpriteSheetFormat, Transparent
    },
};
use crate::{
    DISPLAY_WIDTH, PLAYER_W, PLAYER_H, CRATE_SIZE, GROUND_Y, TOTAL_PLAYER_SPRITE_HEIGHT,
    components::{Player, TwoDimObject}
};

pub struct InitialState;

impl SimpleState for InitialState {
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
        let floating_crate_height = (PLAYER_H + 10) as f32 + GROUND_Y;
        init_crate_sprite(world, &crate_sprite_sheet_handle, DISPLAY_WIDTH - CRATE_SIZE, floating_crate_height);
        init_crate_sprite(world, &crate_sprite_sheet_handle, DISPLAY_WIDTH - 2. * CRATE_SIZE, floating_crate_height);
        init_crate_sprite(world, &crate_sprite_sheet_handle, DISPLAY_WIDTH - 3. * CRATE_SIZE, floating_crate_height);

        world.register::<Player>();
        let sprite_sheet_handle = load_player_sprite_sheet(world);
        init_player(world, &sprite_sheet_handle);

        init_camera(world);
    }
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