/*
    project: CHIP-8 Emulator
    author: Fredrik Reinholdsen
    email: fredrik.reinholdsen@gmail.com
    gitlab: https://gitlab.com/fredrik.reinholdsen

    info:
        An emulator of the CHIP-8 virtual-machine/interpreter from 1970.
        It is essentially an interpreted programming language, designed
        mainly for games. Programs run on a CHIP-8 virtual machine.
*/
pub mod cpu;

use cpu::Cpu;
use ggez_egui::{EguiBackend, egui};
use ggez::{
    event, graphics,
    graphics::DrawParam,
    input::keyboard::{is_key_pressed, KeyCode, KeyMods},
    input::mouse::MouseButton,
    Context, GameResult,
};

const FPS: usize = 60;
const DEFAULT_CLOCK_SPEED: usize = 500;
const ROM: &str = "roms/Breakout [Carmelo Cortez, 1979].ch8";

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const SCREEN_SIZE: (f32, f32) = (800.0, 400.0);
const PIXEL_SIZE: (f32, f32) = (
    SCREEN_SIZE.0 / DISPLAY_WIDTH as f32,
    SCREEN_SIZE.1 / DISPLAY_HEIGHT as f32,
);

// Keys from 0-F that are used to emulate the
// 16-key chip-8 keyboard
const KEYS: [KeyCode; 16] = [
    KeyCode::X,
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::Z,
    KeyCode::C,
    KeyCode::Key4,
    KeyCode::R,
    KeyCode::F,
    KeyCode::V,
];

// Emulates the Chip8's attached 64x32 display

// CHip8 keyboard consists of 16 different keys,
// ranging from 0 to F
pub struct Chip8Display {
    screen: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

// Default implementation for display
impl Default for Chip8Display {
    fn default() -> Self {
        Chip8Display {
            screen: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

}

impl Chip8Display {
    // Clears the screen
    pub fn new() -> Self {
        Chip8Display {
            screen: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    // Clears the screen
    pub fn clear(&mut self) {
        self.screen = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }

    // ggez draw method for drawing the screen to the canvas
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult {
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
    egui_backend: EguiBackend,
    cpu: cpu::Cpu,
    // Number of CPU cycles/ticks executed
    cycles: u128,
    // Step through CPU ticks, one a the time
    show_menu: bool,
}

impl GameState {
    fn new() -> Self {
        let mut cpu = Cpu::new(DEFAULT_CLOCK_SPEED);
        match cpu.load_rom("roms/Breakout [Carmelo Cortez, 1979].ch8") {
            Ok(..) => {}
            Err(e) => panic!("Failed to load ROM!\n{}", e),
        }
        GameState {
            egui_backend: EguiBackend::default(),
            cpu,
            cycles: 0,
            show_menu: false,
        }
    }

    // Draws the egui window
    fn draw_egui(&mut self, ctx: &mut Context) -> GameResult {
        let egui_ctx = self.egui_backend.ctx();
            egui::Window::new("Options").open(&mut self.show_menu).show(&egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Pause").clicked() {
                        self.cpu.set_hold_mode(true);
                    }
                    if ui.button("Play").clicked() {
                        self.cpu.set_hold_mode(false);
                    }
                    if ui.button("Restart").clicked() {
                        self.cpu.reset();
                        self.cpu.load_rom(ROM)
                            .expect("Failed to load ROM!");
                    }
                });
                ui.separator();
                ui.label("CPU Clock Speed:");
                // Slider that changes the clock speed of the emulation
                // thus speeding up or slowing down the game
                ui.add(egui::Slider::new(&mut self.cpu.clock_speed, 50..=2000));
                if ui.button("Quit").clicked() {
                    ggez::event::quit(ctx)
                }
            });
            Ok(())
    }
}

// Implementations of the required ggez methods
impl event::EventHandler<ggez::GameError> for GameState {
    // Updates the state by ticking the CPU,
    // fetching the next, 
    // and instruction and executing that instruction
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ggez::timer::check_update_time(ctx, self.cpu.clock_speed as u32) {
            self.cpu.tick();
            self.cycles += 1;
            let cycles_per_frame = ((1.0 / FPS as f64) / (1.0 / self.cpu.clock_speed as f64)).round() as u128;
            if self.cycles % cycles_per_frame == 0 {
                self.draw(ctx)?;
                self.draw_egui(ctx)?;
            }
        }
        ctx.timer_context.tick();
        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // First we create a canvas that renders to the frame, and clear it to a (sort of) green color
        graphics::clear(ctx, [0.1, 0.1, 0.15, 1.0].into());
        self.cpu.display.draw(ctx)?;
        graphics::draw(ctx, &self.egui_backend, graphics::DrawParam::default())?;
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
        keymods: KeyMods,
        _repeat: bool,
    ) {
        self.egui_backend.input.key_down_event(keycode, keymods);
        match keycode {
            // Toggles the menu
            KeyCode::Return => {
                self.show_menu = !self.show_menu;
            }
            _ => {
                // Lets the CPU know that a key is pressed
                for (i, key) in KEYS.iter().enumerate() {
                    if key == &keycode {
                        self.cpu.pressed_keys[i] = true;
                        return;
                    }
                }
            }
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
    ) {
        for (i, key) in KEYS.iter().enumerate() {
            if key == &keycode {
                self.cpu.pressed_keys[i] = false;
                return;
            }
        }
    }

    // Input methods required for the egui GUI elements
    fn resize_event(&mut self, ctx: &mut ggez::Context, width: f32, height: f32) {	
		self.egui_backend.input.resize_event(width, height);
		let rect = ggez::graphics::Rect::new(0.0, 0.0, width, height);
		ggez::graphics::set_screen_coordinates(ctx, rect).unwrap();
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut ggez::Context, button: ggez::event::MouseButton, _x: f32, _y: f32) {
        self.egui_backend.input.mouse_button_up_event(button);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut ggez::Context, button: ggez::event::MouseButton, _x: f32, _y: f32) {
        self.egui_backend.input.mouse_button_down_event(button);
      }

    fn mouse_wheel_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32) {
        self.egui_backend.input.mouse_wheel_event(x, y);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut ggez::Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.egui_backend.input.mouse_motion_event(x, y);
    }

}

fn main() -> GameResult {
    let (ctx, events_loop) = ggez::ContextBuilder::new("chip8", "Fredrik Reinholdsen")
        .window_setup(ggez::conf::WindowSetup::default().title("CHIP-8 Emulator"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32),
        )
        .build()?;

    // Initialize game state struct and start running game
    let state = GameState::new();
    event::run(ctx, events_loop, state)
}
