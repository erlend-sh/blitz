use std::sync::{Arc, Mutex};
use tao::keyboard::ModifiersState;

use dioxus::{
    core::{ElementId, EventPriority, UserEvent},
    events::KeyboardData,
    native_core::utils::PersistantElementIter,
};
use tao::{dpi::PhysicalPosition, keyboard::Key};

use crate::{Dom, TaoEvent};

#[derive(Default)]
struct EventState {
    modifier_state: ModifiersState,
    cursor_position: PhysicalPosition<f64>,
    focus_iter: Arc<Mutex<PersistantElementIter>>,
    last_focused_id: Option<ElementId>,
}

#[derive(Default)]
pub struct BlitzEventHandler {
    state: EventState,
    queued_events: Vec<UserEvent>,
}

impl BlitzEventHandler {
    pub(crate) fn new(focus_iter: Arc<Mutex<PersistantElementIter>>) -> Self {
        Self {
            state: EventState {
                focus_iter,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // returns weither to force the appliction to redraw
    pub(crate) fn register_event(&mut self, event: &TaoEvent, rdom: &mut Dom) -> bool {
        match event {
            tao::event::Event::NewEvents(_) => (),
            tao::event::Event::WindowEvent {
                window_id: _,
                event,
                ..
            } => {
                match event {
                    tao::event::WindowEvent::Resized(_) => (),
                    tao::event::WindowEvent::Moved(_) => (),
                    tao::event::WindowEvent::CloseRequested => (),
                    tao::event::WindowEvent::Destroyed => (),
                    tao::event::WindowEvent::DroppedFile(_) => (),
                    tao::event::WindowEvent::HoveredFile(_) => (),
                    tao::event::WindowEvent::HoveredFileCancelled => (),
                    tao::event::WindowEvent::ReceivedImeText(_) => (),
                    tao::event::WindowEvent::Focused(_) => (),
                    tao::event::WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                        ..
                    } => {
                        let data = KeyboardData {
                            char_code: event.physical_key.to_scancode().unwrap_or_default(),
                            key: event.logical_key.to_text().unwrap_or_default().to_string(),
                            key_code: translate_key_code(&event.logical_key),
                            alt_key: self.state.modifier_state.alt_key(),
                            ctrl_key: self.state.modifier_state.control_key(),
                            meta_key: self.state.modifier_state.super_key(),
                            shift_key: self.state.modifier_state.shift_key(),
                            locale: "unknown".to_string(),
                            location: match event.location {
                                tao::keyboard::KeyLocation::Standard => 0,
                                tao::keyboard::KeyLocation::Left => 1,
                                tao::keyboard::KeyLocation::Right => 2,
                                tao::keyboard::KeyLocation::Numpad => 3,
                                _ => todo!(),
                            },
                            repeat: event.repeat,
                            // this should return the unicode character
                            which: 0,
                        };

                        // keypress events are only triggered when a key that has text is pressed
                        if let tao::event::ElementState::Pressed = event.state {
                            if event.text.is_some() {
                                self.queued_events.push(UserEvent {
                                    scope_id: None,
                                    priority: EventPriority::Medium,
                                    element: Some(ElementId(1)),
                                    name: "keypress",
                                    data: Arc::new(data.clone()),
                                });
                            }
                            if let Key::Tab = event.logical_key {
                                if let Ok(mut focus_iter) = self.state.focus_iter.lock() {
                                    if let Some(last) = self.state.last_focused_id {
                                        rdom[last].state.focused = false;
                                    }
                                    let mut new;
                                    if self.state.modifier_state.shift_key() {
                                        loop {
                                            new = focus_iter.prev(&rdom);
                                            if rdom[new].state.focusable.0 {
                                                break;
                                            }
                                        }
                                    } else {
                                        loop {
                                            new = focus_iter.next(&rdom);
                                            if rdom[new].state.focusable.0 {
                                                break;
                                            }
                                        }
                                    }
                                    rdom[new].state.focused = true;
                                    self.state.last_focused_id = Some(new);
                                    return true;
                                }
                            }
                        }

                        self.queued_events.push(UserEvent {
                            scope_id: None,
                            priority: EventPriority::Medium,
                            element: self.state.last_focused_id,
                            name: match event.state {
                                tao::event::ElementState::Pressed => "keydown",
                                tao::event::ElementState::Released => "keyup",
                                _ => todo!(),
                            },
                            data: Arc::new(data),
                        });
                    }
                    tao::event::WindowEvent::ModifiersChanged(mods) => {
                        self.state.modifier_state = *mods;
                    }
                    tao::event::WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        ..
                    } => {
                        self.state.cursor_position = *position;
                    }
                    tao::event::WindowEvent::CursorEntered { device_id: _ } => (),
                    tao::event::WindowEvent::CursorLeft { device_id: _ } => (),
                    tao::event::WindowEvent::MouseWheel {
                        device_id: _,
                        delta: _,
                        phase: _,
                        ..
                    } => (),
                    tao::event::WindowEvent::MouseInput {
                        device_id: _,
                        state: _,
                        button: _,
                        ..
                    } => (),
                    tao::event::WindowEvent::TouchpadPressure {
                        device_id: _,
                        pressure: _,
                        stage: _,
                    } => (),
                    tao::event::WindowEvent::AxisMotion {
                        device_id: _,
                        axis: _,
                        value: _,
                    } => (),
                    tao::event::WindowEvent::Touch(_) => (),
                    tao::event::WindowEvent::ScaleFactorChanged {
                        scale_factor: _,
                        new_inner_size: _,
                    } => (),
                    tao::event::WindowEvent::ThemeChanged(_) => (),
                    tao::event::WindowEvent::DecorationsClick => (),
                    _ => (),
                }
            }
            tao::event::Event::DeviceEvent {
                device_id: _,
                event: _,
                ..
            } => (),
            tao::event::Event::UserEvent(_) => (),
            tao::event::Event::MenuEvent {
                window_id: _,
                menu_id: _,
                origin: _,
                ..
            } => (),
            tao::event::Event::TrayEvent {
                bounds: _,
                event: _,
                position: _,
                ..
            } => (),
            tao::event::Event::GlobalShortcutEvent(_) => (),
            tao::event::Event::Suspended => (),
            tao::event::Event::Resumed => (),
            tao::event::Event::MainEventsCleared => (),
            tao::event::Event::RedrawRequested(_) => (),
            tao::event::Event::RedrawEventsCleared => (),
            tao::event::Event::LoopDestroyed => (),
            _ => (),
        }
        false
    }

    pub fn drain_events(&mut self) -> Vec<UserEvent> {
        let mut events = Vec::new();
        std::mem::swap(&mut self.queued_events, &mut events);
        events
    }
}

fn translate_key_code(key: &Key) -> dioxus::events::KeyCode {
    use dioxus::events::KeyCode::*;
    match key {
        Key::Character(char) => match char.to_uppercase().as_str() {
            "A" => A,
            "B" => B,
            "C" => C,
            "D" => D,
            "E" => E,
            "F" => F,
            "G" => G,
            "H" => H,
            "I" => I,
            "J" => J,
            "K" => K,
            "L" => L,
            "M" => M,
            "N" => N,
            "O" => O,
            "P" => P,
            "Q" => Q,
            "R" => R,
            "S" => S,
            "T" => T,
            "U" => U,
            "V" => V,
            "W" => W,
            "X" => X,
            "Y" => Y,
            "Z" => Z,
            "*" => Multiply,
            "−" => Subtract,
            ";" => Semicolon,
            "=" => EqualSign,
            "," => Comma,
            "-" => Dash,
            "." => Period,
            "/" => ForwardSlash,
            "`" => GraveAccent,
            "[" => OpenBracket,
            "\\" => BackSlash,
            "]" => CloseBraket,
            "'" => SingleQuote,
            _ => Unknown,
        },
        Key::Unidentified(_) => Unknown,
        Key::Dead(_) => Unknown,
        Key::Alt => Alt,
        Key::AltGraph => Alt,
        Key::CapsLock => CapsLock,
        Key::Control => Ctrl,
        Key::Fn => Unknown,
        Key::FnLock => Unknown,
        Key::NumLock => NumLock,
        Key::ScrollLock => ScrollLock,
        Key::Shift => Shift,
        Key::Symbol => Unknown,
        Key::SymbolLock => Unknown,
        Key::Hyper => Unknown,
        Key::Super => Unknown,
        Key::Enter => Enter,
        Key::Tab => Tab,
        Key::Space => Space,
        Key::ArrowDown => DownArrow,
        Key::ArrowLeft => LeftArrow,
        Key::ArrowRight => RightArrow,
        Key::ArrowUp => UpArrow,
        Key::End => End,
        Key::Home => Home,
        Key::PageDown => PageDown,
        Key::PageUp => PageUp,
        Key::Backspace => Backspace,
        Key::Clear => Clear,
        Key::Copy => Unknown,
        Key::CrSel => SelectKey,
        Key::Cut => Unknown,
        Key::Delete => Delete,
        Key::EraseEof => Unknown,
        Key::ExSel => Unknown,
        Key::Insert => Insert,
        Key::Paste => Unknown,
        Key::Redo => Unknown,
        Key::Undo => Unknown,
        Key::Accept => Unknown,
        Key::Again => Unknown,
        Key::Attn => Unknown,
        Key::Cancel => Unknown,
        Key::ContextMenu => Unknown,
        Key::Escape => Escape,
        Key::Execute => Unknown,
        Key::Find => Unknown,
        Key::Help => Unknown,
        Key::Pause => Pause,
        Key::Play => Unknown,
        Key::Props => Unknown,
        Key::Select => Unknown,
        Key::ZoomIn => Unknown,
        Key::ZoomOut => Unknown,
        Key::BrightnessDown => Unknown,
        Key::BrightnessUp => Unknown,
        Key::Eject => Unknown,
        Key::LogOff => Unknown,
        Key::Power => Unknown,
        Key::PowerOff => Unknown,
        Key::PrintScreen => Unknown,
        Key::Hibernate => Unknown,
        Key::Standby => Unknown,
        Key::WakeUp => Unknown,
        Key::AllCandidates => Unknown,
        Key::Alphanumeric => Unknown,
        Key::CodeInput => Unknown,
        Key::Compose => Unknown,
        Key::Convert => Unknown,
        Key::FinalMode => Unknown,
        Key::GroupFirst => Unknown,
        Key::GroupLast => Unknown,
        Key::GroupNext => Unknown,
        Key::GroupPrevious => Unknown,
        Key::ModeChange => Unknown,
        Key::NextCandidate => Unknown,
        Key::NonConvert => Unknown,
        Key::PreviousCandidate => Unknown,
        Key::Process => Unknown,
        Key::SingleCandidate => Unknown,
        Key::HangulMode => Unknown,
        Key::HanjaMode => Unknown,
        Key::JunjaMode => Unknown,
        Key::Eisu => Unknown,
        Key::Hankaku => Unknown,
        Key::Hiragana => Unknown,
        Key::HiraganaKatakana => Unknown,
        Key::KanaMode => Unknown,
        Key::KanjiMode => Unknown,
        Key::Katakana => Unknown,
        Key::Romaji => Unknown,
        Key::Zenkaku => Unknown,
        Key::ZenkakuHankaku => Unknown,
        Key::Soft1 => Unknown,
        Key::Soft2 => Unknown,
        Key::Soft3 => Unknown,
        Key::Soft4 => Unknown,
        Key::ChannelDown => Unknown,
        Key::ChannelUp => Unknown,
        Key::Close => Unknown,
        Key::MailForward => Unknown,
        Key::MailReply => Unknown,
        Key::MailSend => Unknown,
        Key::MediaClose => Unknown,
        Key::MediaFastForward => Unknown,
        Key::MediaPause => Unknown,
        Key::MediaPlay => Unknown,
        Key::MediaPlayPause => Unknown,
        Key::MediaRecord => Unknown,
        Key::MediaRewind => Unknown,
        Key::MediaStop => Unknown,
        Key::MediaTrackNext => Unknown,
        Key::MediaTrackPrevious => Unknown,
        Key::New => Unknown,
        Key::Open => Unknown,
        Key::Print => Unknown,
        Key::Save => Unknown,
        Key::SpellCheck => Unknown,
        Key::Key11 => Unknown,
        Key::Key12 => Unknown,
        Key::AudioBalanceLeft => Unknown,
        Key::AudioBalanceRight => Unknown,
        Key::AudioBassBoostDown => Unknown,
        Key::AudioBassBoostToggle => Unknown,
        Key::AudioBassBoostUp => Unknown,
        Key::AudioFaderFront => Unknown,
        Key::AudioFaderRear => Unknown,
        Key::AudioSurroundModeNext => Unknown,
        Key::AudioTrebleDown => Unknown,
        Key::AudioTrebleUp => Unknown,
        Key::AudioVolumeDown => Unknown,
        Key::AudioVolumeUp => Unknown,
        Key::AudioVolumeMute => Unknown,
        Key::MicrophoneToggle => Unknown,
        Key::MicrophoneVolumeDown => Unknown,
        Key::MicrophoneVolumeUp => Unknown,
        Key::MicrophoneVolumeMute => Unknown,
        Key::SpeechCorrectionList => Unknown,
        Key::SpeechInputToggle => Unknown,
        Key::LaunchApplication1 => Unknown,
        Key::LaunchApplication2 => Unknown,
        Key::LaunchCalendar => Unknown,
        Key::LaunchContacts => Unknown,
        Key::LaunchMail => Unknown,
        Key::LaunchMediaPlayer => Unknown,
        Key::LaunchMusicPlayer => Unknown,
        Key::LaunchPhone => Unknown,
        Key::LaunchScreenSaver => Unknown,
        Key::LaunchSpreadsheet => Unknown,
        Key::LaunchWebBrowser => Unknown,
        Key::LaunchWebCam => Unknown,
        Key::LaunchWordProcessor => Unknown,
        Key::BrowserBack => Unknown,
        Key::BrowserFavorites => Unknown,
        Key::BrowserForward => Unknown,
        Key::BrowserHome => Unknown,
        Key::BrowserRefresh => Unknown,
        Key::BrowserSearch => Unknown,
        Key::BrowserStop => Unknown,
        Key::AppSwitch => Unknown,
        Key::Call => Unknown,
        Key::Camera => Unknown,
        Key::CameraFocus => Unknown,
        Key::EndCall => Unknown,
        Key::GoBack => Unknown,
        Key::GoHome => Unknown,
        Key::HeadsetHook => Unknown,
        Key::LastNumberRedial => Unknown,
        Key::Notification => Unknown,
        Key::MannerMode => Unknown,
        Key::VoiceDial => Unknown,
        Key::TV => Unknown,
        Key::TV3DMode => Unknown,
        Key::TVAntennaCable => Unknown,
        Key::TVAudioDescription => Unknown,
        Key::TVAudioDescriptionMixDown => Unknown,
        Key::TVAudioDescriptionMixUp => Unknown,
        Key::TVContentsMenu => Unknown,
        Key::TVDataService => Unknown,
        Key::TVInput => Unknown,
        Key::TVInputComponent1 => Unknown,
        Key::TVInputComponent2 => Unknown,
        Key::TVInputComposite1 => Unknown,
        Key::TVInputComposite2 => Unknown,
        Key::TVInputHDMI1 => Unknown,
        Key::TVInputHDMI2 => Unknown,
        Key::TVInputHDMI3 => Unknown,
        Key::TVInputHDMI4 => Unknown,
        Key::TVInputVGA1 => Unknown,
        Key::TVMediaContext => Unknown,
        Key::TVNetwork => Unknown,
        Key::TVNumberEntry => Unknown,
        Key::TVPower => Unknown,
        Key::TVRadioService => Unknown,
        Key::TVSatellite => Unknown,
        Key::TVSatelliteBS => Unknown,
        Key::TVSatelliteCS => Unknown,
        Key::TVSatelliteToggle => Unknown,
        Key::TVTerrestrialAnalog => Unknown,
        Key::TVTerrestrialDigital => Unknown,
        Key::TVTimer => Unknown,
        Key::AVRInput => Unknown,
        Key::AVRPower => Unknown,
        Key::ColorF0Red => Unknown,
        Key::ColorF1Green => Unknown,
        Key::ColorF2Yellow => Unknown,
        Key::ColorF3Blue => Unknown,
        Key::ColorF4Grey => Unknown,
        Key::ColorF5Brown => Unknown,
        Key::ClosedCaptionToggle => Unknown,
        Key::Dimmer => Unknown,
        Key::DisplaySwap => Unknown,
        Key::DVR => Unknown,
        Key::Exit => Unknown,
        Key::FavoriteClear0 => Unknown,
        Key::FavoriteClear1 => Unknown,
        Key::FavoriteClear2 => Unknown,
        Key::FavoriteClear3 => Unknown,
        Key::FavoriteRecall0 => Unknown,
        Key::FavoriteRecall1 => Unknown,
        Key::FavoriteRecall2 => Unknown,
        Key::FavoriteRecall3 => Unknown,
        Key::FavoriteStore0 => Unknown,
        Key::FavoriteStore1 => Unknown,
        Key::FavoriteStore2 => Unknown,
        Key::FavoriteStore3 => Unknown,
        Key::Guide => Unknown,
        Key::GuideNextDay => Unknown,
        Key::GuidePreviousDay => Unknown,
        Key::Info => Unknown,
        Key::InstantReplay => Unknown,
        Key::Link => Unknown,
        Key::ListProgram => Unknown,
        Key::LiveContent => Unknown,
        Key::Lock => Unknown,
        Key::MediaApps => Unknown,
        Key::MediaAudioTrack => Unknown,
        Key::MediaLast => Unknown,
        Key::MediaSkipBackward => Unknown,
        Key::MediaSkipForward => Unknown,
        Key::MediaStepBackward => Unknown,
        Key::MediaStepForward => Unknown,
        Key::MediaTopMenu => Unknown,
        Key::NavigateIn => Unknown,
        Key::NavigateNext => Unknown,
        Key::NavigateOut => Unknown,
        Key::NavigatePrevious => Unknown,
        Key::NextFavoriteChannel => Unknown,
        Key::NextUserProfile => Unknown,
        Key::OnDemand => Unknown,
        Key::Pairing => Unknown,
        Key::PinPDown => Unknown,
        Key::PinPMove => Unknown,
        Key::PinPToggle => Unknown,
        Key::PinPUp => Unknown,
        Key::PlaySpeedDown => Unknown,
        Key::PlaySpeedReset => Unknown,
        Key::PlaySpeedUp => Unknown,
        Key::RandomToggle => Unknown,
        Key::RcLowBattery => Unknown,
        Key::RecordSpeedNext => Unknown,
        Key::RfBypass => Unknown,
        Key::ScanChannelsToggle => Unknown,
        Key::ScreenModeNext => Unknown,
        Key::Settings => Unknown,
        Key::SplitScreenToggle => Unknown,
        Key::STBInput => Unknown,
        Key::STBPower => Unknown,
        Key::Subtitle => Unknown,
        Key::Teletext => Unknown,
        Key::VideoModeNext => Unknown,
        Key::Wink => Unknown,
        Key::ZoomToggle => Unknown,
        Key::F1 => F1,
        Key::F2 => F2,
        Key::F3 => F3,
        Key::F4 => F4,
        Key::F5 => F5,
        Key::F6 => F6,
        Key::F7 => F7,
        Key::F8 => F8,
        Key::F9 => F9,
        Key::F10 => F10,
        Key::F11 => F11,
        Key::F12 => F12,
        Key::F13 => Unknown,
        Key::F14 => Unknown,
        Key::F15 => Unknown,
        Key::F16 => Unknown,
        Key::F17 => Unknown,
        Key::F18 => Unknown,
        Key::F19 => Unknown,
        Key::F20 => Unknown,
        Key::F21 => Unknown,
        Key::F22 => Unknown,
        Key::F23 => Unknown,
        Key::F24 => Unknown,
        Key::F25 => Unknown,
        Key::F26 => Unknown,
        Key::F27 => Unknown,
        Key::F28 => Unknown,
        Key::F29 => Unknown,
        Key::F30 => Unknown,
        Key::F31 => Unknown,
        Key::F32 => Unknown,
        Key::F33 => Unknown,
        Key::F34 => Unknown,
        Key::F35 => Unknown,
        _ => todo!(),
    }
}
