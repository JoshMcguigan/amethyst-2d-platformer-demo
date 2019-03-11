use amethyst::{
    core::{Transform},
    ecs::{Entities, Join, System, WriteStorage},
    renderer::{Flipped, SpriteRender},
};
use crate::{
    PLAYER_MAX_X_VELOCITY,
    components::{Player, PlayerState}
};

pub struct AnimationSystem;

impl<'s> System<'s> for AnimationSystem {
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