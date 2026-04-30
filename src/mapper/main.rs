use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{env, thread, time::Duration};

use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_DOWN, VK_ESCAPE, VK_LBUTTON, VK_LCONTROL, VK_LEFT, VK_LSHIFT, VK_RBUTTON,
    VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RSHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, SetCursorPos, ShowCursor,
};

const VK_A: i32 = 0x41;
const VK_C: i32 = 0x43;
const VK_D: i32 = 0x44;
const VK_E: i32 = 0x45;
const VK_F: i32 = 0x46;
const VK_Q: i32 = 0x51;
const VK_R: i32 = 0x52;
const VK_S: i32 = 0x53;
const VK_W: i32 = 0x57;

fn main() {
    if has_arg("--help") || has_arg("-h") {
        print_help();
        return;
    }

    let config = MapperConfig::from_args();
    let running = Arc::new(AtomicBool::new(true));
    let stop = running.clone();
    let _ = ctrlc::set_handler(move || stop.store(false, Ordering::SeqCst));

    let client = match Client::connect() {
        Ok(client) => client,
        Err(err) => {
            eprintln!("error: could not connect to ViGEmBus");
            eprintln!("error: run `npm run driver`, accept the Windows prompt, then try again");
            eprintln!("detail: {err}");
            std::process::exit(2);
        }
    };

    let mut target = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
    if let Err(err) = target.plugin().and_then(|_| target.wait_ready()) {
        eprintln!("error: could not start virtual Xbox 360 controller");
        eprintln!("detail: {err}");
        std::process::exit(2);
    }

    println!("status: connected virtual Xbox 360 controller");
    println!("status: press Ctrl+C to stop");

    run_loop(&mut target, config, running);

    let _ = target.update(&XGamepad::default());
    let _ = target.unplug();
    println!("status: stopped");
}

fn run_loop(target: &mut Xbox360Wired<Client>, config: MapperConfig, running: Arc<AtomicBool>) {
    let mut mouse_input = MouseInput::new(config.mouse_capture);
    let mut mouse = MouseState::default();
    let mut tick = 0u32;

    while running.load(Ordering::SeqCst) {
        let (dx, dy) = mouse_input.delta();

        mouse.update(dx, dy, config.mouse_sensitivity);
        let report = build_report(mouse.rx, mouse.ry);
        let _ = target.update(&report);

        tick = tick.wrapping_add(1);
        if tick % 120 == 0 {
            println!(
                "state: wasd={}{}{}{} mouse={},{} buttons={}{}",
                pressed_char(VK_W),
                pressed_char(VK_A),
                pressed_char(VK_S),
                pressed_char(VK_D),
                dx,
                dy,
                if down(VK_LBUTTON.0 as i32) { "L" } else { "-" },
                if down(VK_RBUTTON.0 as i32) { "R" } else { "-" },
            );
        }

        thread::sleep(Duration::from_millis(8));
    }
}

struct MouseInput {
    capture: Option<MouseCapture>,
    last_pos: Option<POINT>,
}

impl MouseInput {
    fn new(capture: bool) -> Self {
        let capture = capture.then(MouseCapture::new);
        let last_pos = if capture.is_some() {
            None
        } else {
            cursor_pos()
        };
        Self { capture, last_pos }
    }

    fn delta(&mut self) -> (i32, i32) {
        if let Some(capture) = &self.capture {
            return capture.delta();
        }

        let current_pos = cursor_pos();
        let delta = match (self.last_pos, current_pos) {
            (Some(last), Some(current)) => (current.x - last.x, current.y - last.y),
            _ => (0, 0),
        };
        self.last_pos = current_pos;
        delta
    }
}

struct MouseCapture {
    center: POINT,
    restore_pos: Option<POINT>,
    _cursor: CursorVisibilityGuard,
}

impl MouseCapture {
    fn new() -> Self {
        let center = POINT {
            x: unsafe { GetSystemMetrics(SM_CXSCREEN) / 2 },
            y: unsafe { GetSystemMetrics(SM_CYSCREEN) / 2 },
        };

        let capture = Self {
            center,
            restore_pos: cursor_pos(),
            _cursor: CursorVisibilityGuard::hide(),
        };
        set_cursor_pos(center);
        capture
    }

    fn delta(&self) -> (i32, i32) {
        let Some(current) = cursor_pos() else {
            return (0, 0);
        };

        let dx = current.x - self.center.x;
        let dy = current.y - self.center.y;
        if dx != 0 || dy != 0 {
            set_cursor_pos(self.center);
        }
        (dx, dy)
    }
}

impl Drop for MouseCapture {
    fn drop(&mut self) {
        if let Some(pos) = self.restore_pos {
            set_cursor_pos(pos);
        }
    }
}

struct CursorVisibilityGuard {
    hide_calls: i32,
}

impl CursorVisibilityGuard {
    fn hide() -> Self {
        let mut hide_calls = 0;
        while unsafe { ShowCursor(false) } >= 0 {
            hide_calls += 1;
        }
        hide_calls += 1;
        Self { hide_calls }
    }
}

impl Drop for CursorVisibilityGuard {
    fn drop(&mut self) {
        for _ in 0..self.hide_calls {
            unsafe {
                ShowCursor(true);
            }
        }
    }
}

fn build_report(mouse_rx: i16, mouse_ry: i16) -> XGamepad {
    let mut buttons = 0u16;

    add_button(
        &mut buttons,
        XButtons::A,
        down(VK_LSHIFT.0 as i32) || down(VK_RSHIFT.0 as i32) || down(VK_SPACE.0 as i32),
    );
    add_button(&mut buttons, XButtons::B, down(VK_ESCAPE.0 as i32));
    add_button(&mut buttons, XButtons::X, down(VK_E));
    add_button(&mut buttons, XButtons::Y, down(VK_R));
    add_button(&mut buttons, XButtons::LB, down(VK_Q));
    add_button(&mut buttons, XButtons::RB, down(VK_F));
    add_button(&mut buttons, XButtons::BACK, down(VK_TAB.0 as i32));
    add_button(&mut buttons, XButtons::START, down(VK_RETURN.0 as i32));
    add_button(
        &mut buttons,
        XButtons::LTHUMB,
        down(VK_LCONTROL.0 as i32) || down(VK_RCONTROL.0 as i32),
    );
    add_button(&mut buttons, XButtons::RTHUMB, down(VK_C));
    add_button(&mut buttons, XButtons::UP, down(VK_UP.0 as i32));
    add_button(&mut buttons, XButtons::DOWN, down(VK_DOWN.0 as i32));
    add_button(&mut buttons, XButtons::LEFT, down(VK_LEFT.0 as i32));
    add_button(&mut buttons, XButtons::RIGHT, down(VK_RIGHT.0 as i32));

    XGamepad {
        buttons: XButtons::from(buttons),
        left_trigger: if down(VK_RBUTTON.0 as i32) {
            u8::MAX
        } else {
            0
        },
        right_trigger: if down(VK_LBUTTON.0 as i32) {
            u8::MAX
        } else {
            0
        },
        thumb_lx: stick_axis((down(VK_D) as i32) - (down(VK_A) as i32)),
        thumb_ly: stick_axis((down(VK_W) as i32) - (down(VK_S) as i32)),
        thumb_rx: mouse_rx,
        thumb_ry: mouse_ry,
    }
}

#[derive(Default)]
struct MouseState {
    rx: i16,
    ry: i16,
}

impl MouseState {
    fn update(&mut self, dx: i32, dy: i32, sensitivity: i32) {
        let target_x = dx.saturating_mul(sensitivity) as f32;
        let target_y = dy.saturating_mul(-sensitivity) as f32;
        self.rx = smooth_axis(self.rx, target_x, dx == 0);
        self.ry = smooth_axis(self.ry, target_y, dy == 0);
    }
}

fn smooth_axis(current: i16, target: f32, idle: bool) -> i16 {
    let current = current as f32;
    let next = if idle {
        current * 0.65
    } else {
        (current * 0.45) + (target * 0.55)
    };

    if next.abs() < 80.0 {
        0
    } else {
        clamp_stick(next.round() as i32)
    }
}

fn add_button(buttons: &mut u16, flag: u16, enabled: bool) {
    if enabled {
        *buttons |= flag;
    }
}

fn down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) < 0 }
}

fn pressed_char(vk: i32) -> char {
    if down(vk) { '1' } else { '0' }
}

fn cursor_pos() -> Option<POINT> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok()? };
    Some(point)
}

fn set_cursor_pos(point: POINT) {
    let _ = unsafe { SetCursorPos(point.x, point.y) };
}

fn stick_axis(direction: i32) -> i16 {
    match direction {
        value if value > 0 => i16::MAX,
        value if value < 0 => i16::MIN,
        _ => 0,
    }
}

fn clamp_stick(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn has_arg(name: &str) -> bool {
    env::args().any(|arg| arg == name)
}

fn print_help() {
    println!(
        "skate-kbm-mapper\n\nCreates a virtual Xbox 360 controller and maps keyboard/mouse input.\n\nOptions:\n  --mouse-sensitivity <number>   Right-stick mouse sensitivity. Default: 500\n  --no-mouse-capture             Do not hide or recenter the Windows cursor\n  -h, --help                     Show help"
    );
}

#[derive(Clone, Copy)]
struct MapperConfig {
    mouse_sensitivity: i32,
    mouse_capture: bool,
}

impl MapperConfig {
    fn from_args() -> Self {
        let mut sensitivity = 500;
        let mut mouse_capture = true;
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--mouse-sensitivity" {
                if let Some(value) = args.next().and_then(|raw| raw.parse::<i32>().ok()) {
                    sensitivity = value.clamp(1, 2000);
                }
            } else if arg == "--no-mouse-capture" {
                mouse_capture = false;
            }
        }

        Self {
            mouse_sensitivity: sensitivity,
            mouse_capture,
        }
    }
}
