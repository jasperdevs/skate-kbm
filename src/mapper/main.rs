use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::{env, mem::size_of, thread, time::Duration};

use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_BACK, VK_DOWN, VK_ESCAPE, VK_LBUTTON, VK_LCONTROL, VK_LEFT, VK_LSHIFT,
    VK_MENU, VK_RBUTTON, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RSHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::Input::{
    GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RID_INPUT, RIDEV_INPUTSINK,
    RIM_TYPEMOUSE, RegisterRawInputDevices,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetCursorPos, GetMessageW, GetSystemMetrics, HWND_MESSAGE, MSG, SM_CXSCREEN,
    SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_SWAPBUTTON, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, SetCursorPos, ShowCursor, WINDOW_EX_STYLE, WINDOW_STYLE, WM_INPUT,
};
use windows::core::w;

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
    if let Err(err) = ctrlc::set_handler(move || stop.store(false, Ordering::SeqCst)) {
        eprintln!("error: could not install Ctrl+C handler");
        eprintln!("detail: {err}");
        std::process::exit(2);
    }

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

    if let Err(err) = target.update(&XGamepad::default()) {
        eprintln!("warning: could not reset virtual controller state: {err}");
    }
    if let Err(err) = target.unplug() {
        eprintln!("warning: could not unplug virtual controller cleanly: {err}");
    }
    println!("status: stopped");
}

fn run_loop(target: &mut Xbox360Wired<Client>, config: MapperConfig, running: Arc<AtomicBool>) {
    let mut mouse_input = MouseInput::new(config.cursor_lock);
    let mut mouse = MouseState::default();
    let mut update_failures = 0u8;
    let mut tick = 0u32;

    println!("status: mouse input {}", mouse_input.mode_name());

    while running.load(Ordering::SeqCst) {
        if control_down() && down(VK_MENU.0 as i32) && down(VK_BACK.0 as i32) {
            println!("status: Ctrl+Alt+Backspace pressed, stopping");
            break;
        }

        let (dx, dy) = mouse_input.delta();

        mouse.update(dx, dy, config.mouse_sensitivity);
        let report = build_report(mouse.rx, mouse.ry);
        if let Err(err) = target.update(&report) {
            update_failures = update_failures.saturating_add(1);
            eprintln!("error: failed to update virtual controller: {err}");
            if update_failures >= 5 {
                eprintln!("error: stopping after repeated virtual controller update failures");
                break;
            }
        } else {
            update_failures = 0;
        }

        tick = tick.wrapping_add(1);
        if config.debug && tick.is_multiple_of(120) {
            println!(
                "debug: wasd={}{}{}{} mouse={},{} buttons={}{} lt={} rt={}",
                pressed_char(VK_W),
                pressed_char(VK_A),
                pressed_char(VK_S),
                pressed_char(VK_D),
                dx,
                dy,
                if down(VK_LBUTTON.0 as i32) { "L" } else { "-" },
                if down(VK_RBUTTON.0 as i32) { "R" } else { "-" },
                if secondary_mouse_down() { "1" } else { "0" },
                if primary_mouse_down() { "1" } else { "0" },
            );
        }

        thread::sleep(Duration::from_millis(8));
    }
}

struct MouseInput {
    source: MouseInputSource,
    raw_cursor_lock_mode: CursorLockMode,
    active_raw_cursor_lock: Option<CursorCapture>,
}

impl MouseInput {
    fn new(cursor_lock: CursorLockMode) -> Self {
        if let Some(raw) = RawMouseInput::new() {
            return Self {
                source: MouseInputSource::Raw(raw),
                raw_cursor_lock_mode: cursor_lock,
                active_raw_cursor_lock: None,
            };
        }

        if cursor_lock != CursorLockMode::Off {
            return Self {
                source: MouseInputSource::Capture(CursorCapture::new()),
                raw_cursor_lock_mode: CursorLockMode::Off,
                active_raw_cursor_lock: None,
            };
        }

        Self {
            source: MouseInputSource::Cursor {
                last_pos: cursor_pos(),
            },
            raw_cursor_lock_mode: CursorLockMode::Off,
            active_raw_cursor_lock: None,
        }
    }

    fn delta(&mut self) -> (i32, i32) {
        if matches!(&self.source, MouseInputSource::Raw(_)) {
            self.update_raw_cursor_lock();
        }

        match &mut self.source {
            MouseInputSource::Raw(raw) => {
                if let Some(lock) = &self.active_raw_cursor_lock {
                    lock.keep_centered();
                }
                raw.delta()
            }
            MouseInputSource::Capture(capture) => capture.delta(),
            MouseInputSource::Cursor { last_pos } => {
                let current_pos = cursor_pos();
                let delta = match (*last_pos, current_pos) {
                    (Some(last), Some(current)) => (current.x - last.x, current.y - last.y),
                    _ => (0, 0),
                };
                *last_pos = current_pos;
                delta
            }
        }
    }

    fn mode_name(&self) -> &'static str {
        match (&self.source, self.raw_cursor_lock_mode) {
            (MouseInputSource::Raw(_), CursorLockMode::HoldButton) => "raw with hold-to-lock cursor",
            (MouseInputSource::Raw(_), CursorLockMode::Always) => "raw with always-on cursor lock",
            (MouseInputSource::Raw(_), CursorLockMode::Off) => "raw",
            (MouseInputSource::Capture(_), _) => "cursor-capture fallback",
            (MouseInputSource::Cursor { .. }, _) => "cursor fallback",
        }
    }

    fn update_raw_cursor_lock(&mut self) {
        let should_lock = match self.raw_cursor_lock_mode {
            CursorLockMode::Always => true,
            CursorLockMode::HoldButton => primary_mouse_down() || secondary_mouse_down(),
            CursorLockMode::Off => false,
        };

        if should_lock {
            if self.active_raw_cursor_lock.is_none() {
                self.active_raw_cursor_lock = Some(CursorCapture::new());
            }
        } else {
            self.active_raw_cursor_lock = None;
        }
    }
}

enum MouseInputSource {
    Raw(RawMouseInput),
    Capture(CursorCapture),
    Cursor { last_pos: Option<POINT> },
}

struct RawMouseInput {
    delta: Arc<Mutex<(i32, i32)>>,
}

impl RawMouseInput {
    fn new() -> Option<Self> {
        let delta = Arc::new(Mutex::new((0, 0)));
        let thread_delta = Arc::clone(&delta);
        let (ready_tx, ready_rx) = mpsc::channel();

        thread::spawn(move || {
            if let Err(err) = raw_input_thread(thread_delta, ready_tx) {
                eprintln!("error: raw mouse input unavailable: {err}");
            }
        });

        match ready_rx.recv_timeout(Duration::from_millis(1500)) {
            Ok(true) => Some(Self { delta }),
            _ => None,
        }
    }

    fn delta(&self) -> (i32, i32) {
        match self.delta.lock() {
            Ok(mut delta) => {
                let current = *delta;
                *delta = (0, 0);
                current
            }
            Err(_) => (0, 0),
        }
    }
}

fn raw_input_thread(
    delta: Arc<Mutex<(i32, i32)>>,
    ready: mpsc::Sender<bool>,
) -> windows::core::Result<()> {
    unsafe {
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("skate-kbm raw input"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )?;

        let device = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x02,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        RegisterRawInputDevices(&[device], size_of::<RAWINPUTDEVICE>() as u32)?;
        let _ = ready.send(true);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_INPUT
                && let Some((dx, dy)) = raw_mouse_delta(HRAWINPUT(msg.lParam.0 as _))
                && let Ok(mut pending) = delta.lock()
            {
                pending.0 = pending.0.saturating_add(dx);
                pending.1 = pending.1.saturating_add(dy);
            }
        }
    }

    Ok(())
}

fn raw_mouse_delta(raw_input: HRAWINPUT) -> Option<(i32, i32)> {
    unsafe {
        let mut size = 0u32;
        GetRawInputData(
            raw_input,
            RID_INPUT,
            None,
            &mut size,
            size_of::<windows::Win32::UI::Input::RAWINPUTHEADER>() as u32,
        );
        if size == 0 {
            return None;
        }

        if size as usize > size_of::<RAWINPUT>() {
            return None;
        }

        let mut input = RAWINPUT::default();
        let read = GetRawInputData(
            raw_input,
            RID_INPUT,
            Some((&mut input as *mut RAWINPUT).cast()),
            &mut size,
            size_of::<windows::Win32::UI::Input::RAWINPUTHEADER>() as u32,
        );
        if read == u32::MAX || read != size {
            return None;
        }

        if input.header.dwType != RIM_TYPEMOUSE.0 {
            return None;
        }

        let mouse = input.data.mouse;
        Some((mouse.lLastX, mouse.lLastY))
    }
}

struct CursorCapture {
    center: POINT,
    restore_pos: Option<POINT>,
    _cursor: CursorVisibilityGuard,
}

impl CursorCapture {
    fn new() -> Self {
        let center = POINT {
            x: virtual_screen_midpoint(SM_XVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CXSCREEN),
            y: virtual_screen_midpoint(SM_YVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_CYSCREEN),
        };

        let capture = Self {
            center,
            restore_pos: cursor_pos(),
            _cursor: CursorVisibilityGuard::hide(),
        };
        if !set_cursor_pos(center) {
            eprintln!("warning: could not center cursor for capture");
        }
        capture
    }

    fn keep_centered(&self) {
        if !set_cursor_pos(self.center) {
            eprintln!("warning: could not keep cursor centered");
        }
    }

    fn delta(&self) -> (i32, i32) {
        let Some(current) = cursor_pos() else {
            return (0, 0);
        };

        let dx = current.x - self.center.x;
        let dy = current.y - self.center.y;
        if (dx != 0 || dy != 0) && !set_cursor_pos(self.center) {
            return (0, 0);
        }
        (dx, dy)
    }
}

impl Drop for CursorCapture {
    fn drop(&mut self) {
        if let Some(pos) = self.restore_pos {
            let _ = set_cursor_pos(pos);
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
        left_trigger: if secondary_mouse_down() { u8::MAX } else { 0 },
        right_trigger: if primary_mouse_down() { u8::MAX } else { 0 },
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

fn set_cursor_pos(point: POINT) -> bool {
    unsafe { SetCursorPos(point.x, point.y).is_ok() }
}

fn virtual_screen_midpoint(
    origin_metric: windows::Win32::UI::WindowsAndMessaging::SYSTEM_METRICS_INDEX,
    size_metric: windows::Win32::UI::WindowsAndMessaging::SYSTEM_METRICS_INDEX,
    fallback_size_metric: windows::Win32::UI::WindowsAndMessaging::SYSTEM_METRICS_INDEX,
) -> i32 {
    let origin = unsafe { GetSystemMetrics(origin_metric) };
    let size = unsafe { GetSystemMetrics(size_metric) };
    if size > 0 {
        origin + (size / 2)
    } else {
        unsafe { GetSystemMetrics(fallback_size_metric) / 2 }
    }
}

fn primary_mouse_down() -> bool {
    let swapped = unsafe { GetSystemMetrics(SM_SWAPBUTTON) != 0 };
    let key = if swapped { VK_RBUTTON } else { VK_LBUTTON };
    down(key.0 as i32)
}

fn secondary_mouse_down() -> bool {
    let swapped = unsafe { GetSystemMetrics(SM_SWAPBUTTON) != 0 };
    let key = if swapped { VK_LBUTTON } else { VK_RBUTTON };
    down(key.0 as i32)
}

fn control_down() -> bool {
    down(VK_LCONTROL.0 as i32) || down(VK_RCONTROL.0 as i32)
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
        "skate-kbm-mapper\n\nCreates a virtual Xbox 360 controller and maps keyboard/mouse input.\n\nOptions:\n  --mouse-sensitivity <number>   Right-stick mouse sensitivity. Default: 500\n  --cursor-lock <mode>           Cursor lock mode: hold, always, off. Default: hold\n  --no-mouse-capture             Alias for --cursor-lock off\n  --debug                        Print live input state\n  -h, --help                     Show help"
    );
}

#[derive(Clone, Copy)]
struct MapperConfig {
    mouse_sensitivity: i32,
    cursor_lock: CursorLockMode,
    debug: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorLockMode {
    HoldButton,
    Always,
    Off,
}

impl MapperConfig {
    fn from_args() -> Self {
        let mut sensitivity = 500;
        let mut cursor_lock = CursorLockMode::HoldButton;
        let mut debug = false;
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--mouse-sensitivity" {
                if let Some(value) = args.next().and_then(|raw| raw.parse::<i32>().ok()) {
                    sensitivity = value.clamp(1, 2000);
                }
            } else if arg == "--cursor-lock" {
                if let Some(value) = args.next() {
                    cursor_lock = match value.as_str() {
                        "always" => CursorLockMode::Always,
                        "off" | "none" | "false" => CursorLockMode::Off,
                        _ => CursorLockMode::HoldButton,
                    };
                }
            } else if arg == "--no-mouse-capture" {
                cursor_lock = CursorLockMode::Off;
            } else if arg == "--debug" {
                debug = true;
            }
        }

        Self {
            mouse_sensitivity: sensitivity,
            cursor_lock,
            debug,
        }
    }
}
