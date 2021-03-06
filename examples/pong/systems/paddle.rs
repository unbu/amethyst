use Paddle;
use amethyst::core::timing::Time;
use amethyst::core::transform::LocalTransform;
use amethyst::ecs::{Fetch, Join, System, WriteStorage};
use amethyst::input::InputHandler;

/// This system is responsible for moving all the paddles according to the user
/// provided input.
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        WriteStorage<'s, Paddle>,
        WriteStorage<'s, LocalTransform>,
        Fetch<'s, Time>,
        Fetch<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (mut paddles, mut transforms, time, input): Self::SystemData) {
        use Side;

        // Iterate over all planks and move them according to the input the user
        // provided.
        for (paddle, transform) in (&mut paddles, &mut transforms).join() {
            let opt_movement = match paddle.side {
                Side::Left => input.axis_value("left_paddle"),
                Side::Right => input.axis_value("right_paddle"),
            };

            if let Some(movement) = opt_movement {
                use ARENA_HEIGHT;
                transform.translation[1] +=
                    paddle.velocity * time.delta_seconds() * movement as f32;

                // We make sure the paddle remains in the arena.
                transform.translation[1] = transform.translation[1]
                    .max(0.0)
                    .min(ARENA_HEIGHT - paddle.height);
            }
        }
    }
}
