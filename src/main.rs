pub mod cpu;

use ggez::{
    event, graphics,
    graphics::{DrawParam, Drawable},
    input::keyboard::{KeyCode, KeyMods},
    Context, GameResult,
};

const FPS: usize = 60;
const CLOCK_SPEED: usize = 2_000_000;
const TICKS_PER_FRAME: usize = FPS / CLOCK_SPEED + 1;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const SCREEN_SIZE: (f32, f32) = (800.0, 400.0);
const PIXEL_SIZE: (f32, f32) = (
    SCREEN_SIZE.0 / DISPLAY_WIDTH as f32,
    SCREEN_SIZE.1 / DISPLAY_HEIGHT as f32,
);

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
                    ).expect("Failed to create pixel mesh!");
                    graphics::draw(ctx, &rect, DrawParam::new()).expect("Failed to draw display!");
                }
            });
        });
        Ok(())
    }
}

struct GameState {
    cpu: cpu::Cpu,
    cycles: u128,
    step_mode: bool,
}

impl GameState {
    fn new() -> Self {
        GameState {
            cpu: cpu::Cpu::new(),
            cycles: 0,
            step_mode: false,
        }
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ggez::timer::check_update_time(ctx, TICKS_PER_FRAME as u32) {
            if !self.step_mode {
                self.cpu.tick();
                self.cycles += 1;
            }
            self.draw(ctx)?;
        }

        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we create a canvas that renders to the frame, and clear it to a (sort of) green color
        graphics::clear(ctx, [0.1, 0.1, 0.15, 1.0].into());
        self.cpu.display.draw(ctx)?;
        //self.cpu.display.draw(&mut canvas);
        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
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
            KeyCode::Key1 => self.cpu.pressed_keys[0] = true,
            KeyCode::Key2 => self.cpu.pressed_keys[1] = true,
            KeyCode::Key3 => self.cpu.pressed_keys[2] = true,
            KeyCode::Key4 => self.cpu.pressed_keys[3] = true,
            KeyCode::Q => self.cpu.pressed_keys[4] = true,
            KeyCode::W => self.cpu.pressed_keys[5] = true,
            KeyCode::E => self.cpu.pressed_keys[6] = true,
            KeyCode::R => self.cpu.pressed_keys[7] = true,
            KeyCode::A => self.cpu.pressed_keys[8] = true,
            KeyCode::S => self.cpu.pressed_keys[9] = true,
            KeyCode::D => self.cpu.pressed_keys[10] = true,
            KeyCode::F => self.cpu.pressed_keys[11] = true,
            KeyCode::Z => self.cpu.pressed_keys[12] = true,
            KeyCode::X => self.cpu.pressed_keys[13] = true,
            KeyCode::C => self.cpu.pressed_keys[14] = true,
            KeyCode::V => self.cpu.pressed_keys[15] = true,
            KeyCode::M => {
                if self.step_mode {
                    println!("Exiting step mode!");
                    self.step_mode = false;
                } else {
                    println!("Entering step mode! Press ENTER/RETURN to step forward, or press S again to exit step mode.");
                    self.step_mode = true;
                }
            },
            KeyCode::Return =>  {
                if self.step_mode {
                    self.cpu.tick();
                    self.cycles += 1;
                }
            }
            _ => {},
        }
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _mods: KeyMods,
    ) {
        match keycode {
            KeyCode::Key1 => self.cpu.pressed_keys[0] = false,
            KeyCode::Key2 => self.cpu.pressed_keys[1] = false,
            KeyCode::Key3 => self.cpu.pressed_keys[2] = false,
            KeyCode::Key4 => self.cpu.pressed_keys[3] = false,
            KeyCode::Q => self.cpu.pressed_keys[4] = false,
            KeyCode::W => self.cpu.pressed_keys[5] = false,
            KeyCode::E => self.cpu.pressed_keys[6] = false,
            KeyCode::R => self.cpu.pressed_keys[7] = false,
            KeyCode::A => self.cpu.pressed_keys[8] = false,
            KeyCode::S => self.cpu.pressed_keys[9] = false,
            KeyCode::D => self.cpu.pressed_keys[10] = false,
            KeyCode::F => self.cpu.pressed_keys[11] = false,
            KeyCode::Z => self.cpu.pressed_keys[12] = false,
            KeyCode::X => self.cpu.pressed_keys[13] = false,
            KeyCode::C => self.cpu.pressed_keys[14] = false,
            KeyCode::V => self.cpu.pressed_keys[15] = false,
            _ => {},
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
    state.cpu.stack_print();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}
