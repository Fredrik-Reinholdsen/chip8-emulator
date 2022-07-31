pub mod cpu;

use cpu::Cpu;
use ggez::{
    event, graphics,
    graphics::{DrawParam, Drawable},
    input::keyboard::{is_key_pressed, KeyCode, KeyMods},
    Context, GameResult,
};

const FPS: usize = 60;
const CLOCK_SPEED: f64 = 1e3;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const SCREEN_SIZE: (f32, f32) = (800.0, 400.0);
const PIXEL_SIZE: (f32, f32) = (
    SCREEN_SIZE.0 / DISPLAY_WIDTH as f32,
    SCREEN_SIZE.1 / DISPLAY_HEIGHT as f32,
);
const KEYS: [KeyCode; 16] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::R,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::C,
    KeyCode::V,
];

// Emulates the Chip8's attached 64x32 display

// CHip8 keyboard consists of 16 different keys,
// ranging from 0 to F
pub struct Chip8Display {
    screen: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Chip8Display {
    // Clears the screen
    pub fn new() -> Self {
        Chip8Display {
            screen: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }
    pub fn clear(&mut self) {
        self.screen = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult {
        // Clears the terminal before printing the display
        (0..DISPLAY_HEIGHT).into_iter().for_each(|row| {
            (0..DISPLAY_WIDTH).into_iter().for_each(|col| {
                if self.screen[row][col] {
                    let x = PIXEL_SIZE.1 * col as f32;
                    let y = PIXEL_SIZE.0 * row as f32;
                    let rect = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        [x, y, PIXEL_SIZE.0, PIXEL_SIZE.1].into(),
                        [1.0, 1.0, 1.0, 1.0].into(),
                    )
                    .expect("Failed to create pixel mesh!");
                    graphics::draw(ctx, &rect, DrawParam::new()).expect("Failed to draw display!");
                }
            });
        });
        Ok(())
    }
}

struct GameState {
    cpu: cpu::Cpu,
    // Number of CPU cycles/ticks executed
    cycles: u128,
    // Step through CPU ticks, one a the time
    step_mode: bool,
}

impl GameState {
    fn new() -> Self {
        let mut cpu = Cpu::new(CLOCK_SPEED);
        match cpu.load_rom("roms/Pong (alt).ch8") {
            Ok(..) => {}
            Err(e) => panic!("Failed to load ROM!\n{}", e),
        }
        GameState {
            cpu,
            cycles: 0,
            step_mode: false,
        }
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ggez::timer::check_update_time(ctx, CLOCK_SPEED as u32) {
            for (i, key) in KEYS.iter().enumerate() {
                self.cpu.pressed_keys[i] = is_key_pressed(ctx, *key);
            }
            if !self.step_mode {
                self.cpu.tick();
                self.cycles += 1;
            }
            let cycles_per_frame = ((1.0 / FPS as f64) / (1.0 / CLOCK_SPEED)).round() as u128;
            if self.cycles % cycles_per_frame == 0 {
                self.draw(ctx)?;
            }
        }
        ggez::timer::yield_now();
        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we create a canvas that renders to the frame, and clear it to a (sort of) green color
        graphics::clear(ctx, [0.1, 0.1, 0.15, 1.0].into());
        self.cpu.display.draw(ctx)?;
        //self.cpu.display.draw(&mut canvas);
        graphics::present(ctx)
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _mods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::M => {
                if self.step_mode {
                    println!("Exiting step mode!");
                    self.step_mode = false;
                } else {
                    println!("Entering step mode! Press ENTER/RETURN to step forward, or press S again to exit step mode.");
                    self.step_mode = true;
                }
            }
            KeyCode::Return => {
                if self.step_mode {
                    self.cpu.tick();
                    self.cycles += 1;
                    println!("{}", self.cycles);
                }
            }
            _ => {}
        }
    }
}

fn main() -> GameResult {
    let (ctx, events_loop) = ggez::ContextBuilder::new("chip8", "Fredrik Reinholdsen")
        .window_setup(ggez::conf::WindowSetup::default().title("Chip8 Emulator!"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32),
        )
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = GameState::new();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}
