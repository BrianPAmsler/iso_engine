use std::collections::HashMap;

use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(pub(in crate::engine) PhysicalKey);

#[allow(non_upper_case_globals, reason="These globals are a wrapper for an enum, so they should be named like enum variants.")]
impl Key {
    pub const Backquote: Key = Key(PhysicalKey::Code(KeyCode::Backquote));
    pub const Backslash: Key = Key(PhysicalKey::Code(KeyCode::Backslash));
    pub const BracketLeft: Key = Key(PhysicalKey::Code(KeyCode::BracketLeft));
    pub const BracketRight: Key = Key(PhysicalKey::Code(KeyCode::BracketRight));
    pub const Comma: Key = Key(PhysicalKey::Code(KeyCode::Comma));
    pub const Digit0: Key = Key(PhysicalKey::Code(KeyCode::Digit0));
    pub const Digit1: Key = Key(PhysicalKey::Code(KeyCode::Digit1));
    pub const Digit2: Key = Key(PhysicalKey::Code(KeyCode::Digit2));
    pub const Digit3: Key = Key(PhysicalKey::Code(KeyCode::Digit3));
    pub const Digit4: Key = Key(PhysicalKey::Code(KeyCode::Digit4));
    pub const Digit5: Key = Key(PhysicalKey::Code(KeyCode::Digit5));
    pub const Digit6: Key = Key(PhysicalKey::Code(KeyCode::Digit6));
    pub const Digit7: Key = Key(PhysicalKey::Code(KeyCode::Digit7));
    pub const Digit8: Key = Key(PhysicalKey::Code(KeyCode::Digit8));
    pub const Digit9: Key = Key(PhysicalKey::Code(KeyCode::Digit9));
    pub const Equal: Key = Key(PhysicalKey::Code(KeyCode::Equal));
    pub const IntlBackslash: Key = Key(PhysicalKey::Code(KeyCode::IntlBackslash));
    pub const IntlRo: Key = Key(PhysicalKey::Code(KeyCode::IntlRo));
    pub const IntlYen: Key = Key(PhysicalKey::Code(KeyCode::IntlYen));
    pub const KeyA: Key = Key(PhysicalKey::Code(KeyCode::KeyA));
    pub const KeyB: Key = Key(PhysicalKey::Code(KeyCode::KeyB));
    pub const KeyC: Key = Key(PhysicalKey::Code(KeyCode::KeyC));
    pub const KeyD: Key = Key(PhysicalKey::Code(KeyCode::KeyD));
    pub const KeyE: Key = Key(PhysicalKey::Code(KeyCode::KeyE));
    pub const KeyF: Key = Key(PhysicalKey::Code(KeyCode::KeyF));
    pub const KeyG: Key = Key(PhysicalKey::Code(KeyCode::KeyG));
    pub const KeyH: Key = Key(PhysicalKey::Code(KeyCode::KeyH));
    pub const KeyI: Key = Key(PhysicalKey::Code(KeyCode::KeyI));
    pub const KeyJ: Key = Key(PhysicalKey::Code(KeyCode::KeyJ));
    pub const KeyK: Key = Key(PhysicalKey::Code(KeyCode::KeyK));
    pub const KeyL: Key = Key(PhysicalKey::Code(KeyCode::KeyL));
    pub const KeyM: Key = Key(PhysicalKey::Code(KeyCode::KeyM));
    pub const KeyN: Key = Key(PhysicalKey::Code(KeyCode::KeyN));
    pub const KeyO: Key = Key(PhysicalKey::Code(KeyCode::KeyO));
    pub const KeyP: Key = Key(PhysicalKey::Code(KeyCode::KeyP));
    pub const KeyQ: Key = Key(PhysicalKey::Code(KeyCode::KeyQ));
    pub const KeyR: Key = Key(PhysicalKey::Code(KeyCode::KeyR));
    pub const KeyS: Key = Key(PhysicalKey::Code(KeyCode::KeyS));
    pub const KeyT: Key = Key(PhysicalKey::Code(KeyCode::KeyT));
    pub const KeyU: Key = Key(PhysicalKey::Code(KeyCode::KeyU));
    pub const KeyV: Key = Key(PhysicalKey::Code(KeyCode::KeyV));
    pub const KeyW: Key = Key(PhysicalKey::Code(KeyCode::KeyW));
    pub const KeyX: Key = Key(PhysicalKey::Code(KeyCode::KeyX));
    pub const KeyY: Key = Key(PhysicalKey::Code(KeyCode::KeyY));
    pub const KeyZ: Key = Key(PhysicalKey::Code(KeyCode::KeyZ));
    pub const Minus: Key = Key(PhysicalKey::Code(KeyCode::Minus));
    pub const Period: Key = Key(PhysicalKey::Code(KeyCode::Period));
    pub const Quote: Key = Key(PhysicalKey::Code(KeyCode::Quote));
    pub const Semicolon: Key = Key(PhysicalKey::Code(KeyCode::Semicolon));
    pub const Slash: Key = Key(PhysicalKey::Code(KeyCode::Slash));
    pub const AltLeft: Key = Key(PhysicalKey::Code(KeyCode::AltLeft));
    pub const AltRight: Key = Key(PhysicalKey::Code(KeyCode::AltRight));
    pub const Backspace: Key = Key(PhysicalKey::Code(KeyCode::Backspace));
    pub const CapsLock: Key = Key(PhysicalKey::Code(KeyCode::CapsLock));
    pub const ContextMenu: Key = Key(PhysicalKey::Code(KeyCode::ContextMenu));
    pub const ControlLeft: Key = Key(PhysicalKey::Code(KeyCode::ControlLeft));
    pub const ControlRight: Key = Key(PhysicalKey::Code(KeyCode::ControlRight));
    pub const Enter: Key = Key(PhysicalKey::Code(KeyCode::Enter));
    pub const SuperLeft: Key = Key(PhysicalKey::Code(KeyCode::SuperLeft));
    pub const SuperRight: Key = Key(PhysicalKey::Code(KeyCode::SuperRight));
    pub const ShiftLeft: Key = Key(PhysicalKey::Code(KeyCode::ShiftLeft));
    pub const ShiftRight: Key = Key(PhysicalKey::Code(KeyCode::ShiftRight));
    pub const Space: Key = Key(PhysicalKey::Code(KeyCode::Space));
    pub const Tab: Key = Key(PhysicalKey::Code(KeyCode::Tab));
    pub const Convert: Key = Key(PhysicalKey::Code(KeyCode::Convert));
    pub const KanaMode: Key = Key(PhysicalKey::Code(KeyCode::KanaMode));
    pub const Lang1: Key = Key(PhysicalKey::Code(KeyCode::Lang1));
    pub const Lang2: Key = Key(PhysicalKey::Code(KeyCode::Lang2));
    pub const Lang3: Key = Key(PhysicalKey::Code(KeyCode::Lang3));
    pub const Lang4: Key = Key(PhysicalKey::Code(KeyCode::Lang4));
    pub const Lang5: Key = Key(PhysicalKey::Code(KeyCode::Lang5));
    pub const NonConvert: Key = Key(PhysicalKey::Code(KeyCode::NonConvert));
    pub const Delete: Key = Key(PhysicalKey::Code(KeyCode::Delete));
    pub const End: Key = Key(PhysicalKey::Code(KeyCode::End));
    pub const Help: Key = Key(PhysicalKey::Code(KeyCode::Help));
    pub const Home: Key = Key(PhysicalKey::Code(KeyCode::Home));
    pub const Insert: Key = Key(PhysicalKey::Code(KeyCode::Insert));
    pub const PageDown: Key = Key(PhysicalKey::Code(KeyCode::PageDown));
    pub const PageUp: Key = Key(PhysicalKey::Code(KeyCode::PageUp));
    pub const ArrowDown: Key = Key(PhysicalKey::Code(KeyCode::ArrowDown));
    pub const ArrowLeft: Key = Key(PhysicalKey::Code(KeyCode::ArrowLeft));
    pub const ArrowRight: Key = Key(PhysicalKey::Code(KeyCode::ArrowRight));
    pub const ArrowUp: Key = Key(PhysicalKey::Code(KeyCode::ArrowUp));
    pub const NumLock: Key = Key(PhysicalKey::Code(KeyCode::NumLock));
    pub const Numpad0: Key = Key(PhysicalKey::Code(KeyCode::Numpad0));
    pub const Numpad1: Key = Key(PhysicalKey::Code(KeyCode::Numpad1));
    pub const Numpad2: Key = Key(PhysicalKey::Code(KeyCode::Numpad2));
    pub const Numpad3: Key = Key(PhysicalKey::Code(KeyCode::Numpad3));
    pub const Numpad4: Key = Key(PhysicalKey::Code(KeyCode::Numpad4));
    pub const Numpad5: Key = Key(PhysicalKey::Code(KeyCode::Numpad5));
    pub const Numpad6: Key = Key(PhysicalKey::Code(KeyCode::Numpad6));
    pub const Numpad7: Key = Key(PhysicalKey::Code(KeyCode::Numpad7));
    pub const Numpad8: Key = Key(PhysicalKey::Code(KeyCode::Numpad8));
    pub const Numpad9: Key = Key(PhysicalKey::Code(KeyCode::Numpad9));
    pub const NumpadAdd: Key = Key(PhysicalKey::Code(KeyCode::NumpadAdd));
    pub const NumpadBackspace: Key = Key(PhysicalKey::Code(KeyCode::NumpadBackspace));
    pub const NumpadClear: Key = Key(PhysicalKey::Code(KeyCode::NumpadClear));
    pub const NumpadClearEntry: Key = Key(PhysicalKey::Code(KeyCode::NumpadClearEntry));
    pub const NumpadComma: Key = Key(PhysicalKey::Code(KeyCode::NumpadComma));
    pub const NumpadDecimal: Key = Key(PhysicalKey::Code(KeyCode::NumpadDecimal));
    pub const NumpadDivide: Key = Key(PhysicalKey::Code(KeyCode::NumpadDivide));
    pub const NumpadEnter: Key = Key(PhysicalKey::Code(KeyCode::NumpadEnter));
    pub const NumpadEqual: Key = Key(PhysicalKey::Code(KeyCode::NumpadEqual));
    pub const NumpadHash: Key = Key(PhysicalKey::Code(KeyCode::NumpadHash));
    pub const NumpadMemoryAdd: Key = Key(PhysicalKey::Code(KeyCode::NumpadMemoryAdd));
    pub const NumpadMemoryClear: Key = Key(PhysicalKey::Code(KeyCode::NumpadMemoryClear));
    pub const NumpadMemoryRecall: Key = Key(PhysicalKey::Code(KeyCode::NumpadMemoryRecall));
    pub const NumpadMemoryStore: Key = Key(PhysicalKey::Code(KeyCode::NumpadMemoryStore));
    pub const NumpadMemorySubtract: Key = Key(PhysicalKey::Code(KeyCode::NumpadMemorySubtract));
    pub const NumpadMultiply: Key = Key(PhysicalKey::Code(KeyCode::NumpadMultiply));
    pub const NumpadParenLeft: Key = Key(PhysicalKey::Code(KeyCode::NumpadParenLeft));
    pub const NumpadParenRight: Key = Key(PhysicalKey::Code(KeyCode::NumpadParenRight));
    pub const NumpadStar: Key = Key(PhysicalKey::Code(KeyCode::NumpadStar));
    pub const NumpadSubtract: Key = Key(PhysicalKey::Code(KeyCode::NumpadSubtract));
    pub const Escape: Key = Key(PhysicalKey::Code(KeyCode::Escape));
    pub const Fn: Key = Key(PhysicalKey::Code(KeyCode::Fn));
    pub const FnLock: Key = Key(PhysicalKey::Code(KeyCode::FnLock));
    pub const PrintScreen: Key = Key(PhysicalKey::Code(KeyCode::PrintScreen));
    pub const ScrollLock: Key = Key(PhysicalKey::Code(KeyCode::ScrollLock));
    pub const Pause: Key = Key(PhysicalKey::Code(KeyCode::Pause));
    pub const BrowserBack: Key = Key(PhysicalKey::Code(KeyCode::BrowserBack));
    pub const BrowserFavorites: Key = Key(PhysicalKey::Code(KeyCode::BrowserFavorites));
    pub const BrowserForward: Key = Key(PhysicalKey::Code(KeyCode::BrowserForward));
    pub const BrowserHome: Key = Key(PhysicalKey::Code(KeyCode::BrowserHome));
    pub const BrowserRefresh: Key = Key(PhysicalKey::Code(KeyCode::BrowserRefresh));
    pub const BrowserSearch: Key = Key(PhysicalKey::Code(KeyCode::BrowserSearch));
    pub const BrowserStop: Key = Key(PhysicalKey::Code(KeyCode::BrowserStop));
    pub const Eject: Key = Key(PhysicalKey::Code(KeyCode::Eject));
    pub const LaunchApp1: Key = Key(PhysicalKey::Code(KeyCode::LaunchApp1));
    pub const LaunchApp2: Key = Key(PhysicalKey::Code(KeyCode::LaunchApp2));
    pub const LaunchMail: Key = Key(PhysicalKey::Code(KeyCode::LaunchMail));
    pub const MediaPlayPause: Key = Key(PhysicalKey::Code(KeyCode::MediaPlayPause));
    pub const MediaSelect: Key = Key(PhysicalKey::Code(KeyCode::MediaSelect));
    pub const MediaStop: Key = Key(PhysicalKey::Code(KeyCode::MediaStop));
    pub const MediaTrackNext: Key = Key(PhysicalKey::Code(KeyCode::MediaTrackNext));
    pub const MediaTrackPrevious: Key = Key(PhysicalKey::Code(KeyCode::MediaTrackPrevious));
    pub const Power: Key = Key(PhysicalKey::Code(KeyCode::Power));
    pub const Sleep: Key = Key(PhysicalKey::Code(KeyCode::Sleep));
    pub const AudioVolumeDown: Key = Key(PhysicalKey::Code(KeyCode::AudioVolumeDown));
    pub const AudioVolumeMute: Key = Key(PhysicalKey::Code(KeyCode::AudioVolumeMute));
    pub const AudioVolumeUp: Key = Key(PhysicalKey::Code(KeyCode::AudioVolumeUp));
    pub const WakeUp: Key = Key(PhysicalKey::Code(KeyCode::WakeUp));
    pub const Meta: Key = Key(PhysicalKey::Code(KeyCode::Meta));
    pub const Hyper: Key = Key(PhysicalKey::Code(KeyCode::Hyper));
    pub const Turbo: Key = Key(PhysicalKey::Code(KeyCode::Turbo));
    pub const Abort: Key = Key(PhysicalKey::Code(KeyCode::Abort));
    pub const Resume: Key = Key(PhysicalKey::Code(KeyCode::Resume));
    pub const Suspend: Key = Key(PhysicalKey::Code(KeyCode::Suspend));
    pub const Again: Key = Key(PhysicalKey::Code(KeyCode::Again));
    pub const Copy: Key = Key(PhysicalKey::Code(KeyCode::Copy));
    pub const Cut: Key = Key(PhysicalKey::Code(KeyCode::Cut));
    pub const Find: Key = Key(PhysicalKey::Code(KeyCode::Find));
    pub const Open: Key = Key(PhysicalKey::Code(KeyCode::Open));
    pub const Paste: Key = Key(PhysicalKey::Code(KeyCode::Paste));
    pub const Props: Key = Key(PhysicalKey::Code(KeyCode::Props));
    pub const Select: Key = Key(PhysicalKey::Code(KeyCode::Select));
    pub const Undo: Key = Key(PhysicalKey::Code(KeyCode::Undo));
    pub const Hiragana: Key = Key(PhysicalKey::Code(KeyCode::Hiragana));
    pub const Katakana: Key = Key(PhysicalKey::Code(KeyCode::Katakana));
    pub const F1: Key = Key(PhysicalKey::Code(KeyCode::F1));
    pub const F2: Key = Key(PhysicalKey::Code(KeyCode::F2));
    pub const F3: Key = Key(PhysicalKey::Code(KeyCode::F3));
    pub const F4: Key = Key(PhysicalKey::Code(KeyCode::F4));
    pub const F5: Key = Key(PhysicalKey::Code(KeyCode::F5));
    pub const F6: Key = Key(PhysicalKey::Code(KeyCode::F6));
    pub const F7: Key = Key(PhysicalKey::Code(KeyCode::F7));
    pub const F8: Key = Key(PhysicalKey::Code(KeyCode::F8));
    pub const F9: Key = Key(PhysicalKey::Code(KeyCode::F9));
    pub const F10: Key = Key(PhysicalKey::Code(KeyCode::F10));
    pub const F11: Key = Key(PhysicalKey::Code(KeyCode::F11));
    pub const F12: Key = Key(PhysicalKey::Code(KeyCode::F12));
    pub const F13: Key = Key(PhysicalKey::Code(KeyCode::F13));
    pub const F14: Key = Key(PhysicalKey::Code(KeyCode::F14));
    pub const F15: Key = Key(PhysicalKey::Code(KeyCode::F15));
    pub const F16: Key = Key(PhysicalKey::Code(KeyCode::F16));
    pub const F17: Key = Key(PhysicalKey::Code(KeyCode::F17));
    pub const F18: Key = Key(PhysicalKey::Code(KeyCode::F18));
    pub const F19: Key = Key(PhysicalKey::Code(KeyCode::F19));
    pub const F20: Key = Key(PhysicalKey::Code(KeyCode::F20));
    pub const F21: Key = Key(PhysicalKey::Code(KeyCode::F21));
    pub const F22: Key = Key(PhysicalKey::Code(KeyCode::F22));
    pub const F23: Key = Key(PhysicalKey::Code(KeyCode::F23));
    pub const F24: Key = Key(PhysicalKey::Code(KeyCode::F24));
    pub const F25: Key = Key(PhysicalKey::Code(KeyCode::F25));
    pub const F26: Key = Key(PhysicalKey::Code(KeyCode::F26));
    pub const F27: Key = Key(PhysicalKey::Code(KeyCode::F27));
    pub const F28: Key = Key(PhysicalKey::Code(KeyCode::F28));
    pub const F29: Key = Key(PhysicalKey::Code(KeyCode::F29));
    pub const F30: Key = Key(PhysicalKey::Code(KeyCode::F30));
    pub const F31: Key = Key(PhysicalKey::Code(KeyCode::F31));
    pub const F32: Key = Key(PhysicalKey::Code(KeyCode::F32));
    pub const F33: Key = Key(PhysicalKey::Code(KeyCode::F33));
    pub const F34: Key = Key(PhysicalKey::Code(KeyCode::F34));
    pub const F35: Key = Key(PhysicalKey::Code(KeyCode::F35));

    /// Returns a native key code for the target os.
    /// Obviously this is not cross-platform.
    pub const fn native_key_code(code: u32) -> Key {
        #[cfg(target_os = "windows")]
        let native_key_code = winit::keyboard::NativeKeyCode::Windows(code as u16);
        #[cfg(target_os = "linux")]
        let native_key_code = winit::keyboard::NativeKeyCode::Xkb(code);
        #[cfg(target_os = "macos")]
        let native_key_code = winit::keyboard::NativeKeyCode::MacOS(code as u16);
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        let native_key_code = winit::keyboard::NativeKeyCode::Unidentified;
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        eprintln!("Other key codes not supported on this os.");
        Key(PhysicalKey::Unidentified(native_key_code))
    }

    pub fn physical_key(&self) -> PhysicalKey {
        self.0
    }
}

const MOUSE_BUTTON_COUNT: usize = 20; // 20 is probably way overkill, but whatever.
/// Line Height used to convert pixels into lines for mouse scroll events.
pub const LINE_HEIGHT: f64 = 24.0;

pub mod mouse_buttons {
    pub const LEFT: u32 = 1;
    pub const RIGHT: u32 = 2;
    pub const MIDDLE: u32 = 3;
    pub const BACK: u32 = 4;
    pub const FORWARD: u32 = 5;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KeyState {
    pub is_down: bool,
    pub press: bool,
    pub release: bool
}

pub struct Input {
    keys: HashMap<Key, KeyState>,
    mouse_buttons: Box<[KeyState]>,
    scroll_x: f64,
    scroll_y: f64
}

impl Input {
    pub fn new() -> Input {
        let keys = HashMap::new();
        let mouse_buttons = Box::new([KeyState::default(); MOUSE_BUTTON_COUNT]);
        Input { keys, mouse_buttons, scroll_x: 0.0, scroll_y: 0.0 }
    }

    pub fn get_mouse_button_state(&self, button: u32) -> KeyState {
        if button as usize > self.mouse_buttons.len() {
            panic!("Invalid mouse button.");
        }

        self.mouse_buttons[button as usize]
    }

    pub(in crate::engine) fn modify_mouse_button_state(&mut self, button: u32) -> &mut KeyState {
        if button as usize > self.mouse_buttons.len() {
            panic!("Invalid mouse button.");
        }

        &mut self.mouse_buttons[button as usize]
    }

    pub fn get_key_state(&self, key: Key) -> KeyState {
        self.keys.get(&key).copied().unwrap_or_default()
    }

    pub(in crate::engine) fn modify_key_state(&mut self, key: Key) -> &mut KeyState {
        self.keys.entry(key).or_default()
    }

    pub(in crate::engine) fn reset(&mut self) {
        for state in self.keys.values_mut() {
            state.press = false;
            state.release = false;
        }
        
        for state in &mut self.mouse_buttons[..] {
            state.press = false;
            state.release = false;
        }
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
    }

    pub fn get_scroll_x(&self) -> f64 {
        self.scroll_x
    }

    pub fn get_scroll_y(&self) -> f64 {
        self.scroll_y
    }

    pub(in crate::engine) fn add_scroll_delta(&mut self, x: f64, y: f64) {
        self.scroll_x += x;
        self.scroll_y += y;
    }
}

impl Default for Input {
    fn default() -> Self {
        Input::new()
    }
}