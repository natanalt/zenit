//! The developer's UI implementation, also just called DevUi
//!
//! The DevUi code consists of Dear ImGui platform code and any necessary interactions on
//! the renderer side of things.
//!
//! Due to things like dependency issues, this started life as a fork of the imgui-winit-support
//! library, version 0.11. Since then I decided to just kinda adapt the whole thing into the
//! engine, cause why not.
//!
//! https://github.com/imgui-rs/imgui-rs/tree/main/imgui-winit-support
//!
//! There's still a lot that this library doesn't properly support that I'd like to support
//! eventually (and maybe contribute to upstream bindings). At least one thing I've noticed is
//! lacking IME support, which I'd like to eventually support.
//! 

use crate::{
    engine::{EngineContext, FrameHistory, FrameTiming},
    entities::Component,
    graphics::{
        imgui_renderer::{ImguiFrame, ImguiRenderData, ImguiTexture},
        TextureDescriptor, TextureWriteDescriptor,
    },
    scene::EngineBorrow,
};
use glam::*;
use imgui::{self, BackendFlags, ConfigFlags, Key, TextureId};
use log::*;
use std::{cmp::Ordering, sync::Arc, time::Duration};
use winit::{
    dpi::LogicalPosition,
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase,
        VirtualKeyCode, WindowEvent,
    },
    window::{CursorIcon as MouseCursor, Window},
};

#[derive(Debug)]
pub struct DevUi {
    imgui: imgui::Context,
    window: Arc<Window>,
    current_cursor_cache: Option<imgui::MouseCursor>,

    font_needs_update: bool,
    texture_id_accumulator: usize,
}

impl DevUi {
    /// Initializes DevUi and the underlying Dear ImGui context.
    ///
    /// The ImGui initialization includes setting all backend flags for the renderer as well.
    pub fn new(engine: &EngineContext) -> Self {
        let globals = engine.globals.read();

        let window = globals.get::<Arc<Window>>().clone();
        let scale_factor = window.scale_factor() as f32;
        let logical_size = window.inner_size().to_logical::<f32>(scale_factor as _);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui.set_platform_name(Some(format!("zenit {}", crate::VERSION)));
        imgui.set_renderer_name(Some(format!("zenit {}", crate::VERSION)));
        imgui.fonts().tex_id = TextureId::new(0);

        let io = imgui.io_mut();
        io.display_framebuffer_scale = [scale_factor, scale_factor];
        io.display_size = [logical_size.width, logical_size.height];
        io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
        io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
        io.backend_flags
            .insert(BackendFlags::RENDERER_HAS_VTX_OFFSET);

        Self {
            imgui,
            window,
            current_cursor_cache: None,
            font_needs_update: true,
            texture_id_accumulator: 0,
        }
    }

    /// Performs all per-frame tasks of the DevUi, including notifying Dear ImGui of any window
    /// events, running all UI widgets, and sending the rendered UI mesh to the renderer.
    pub fn process(&mut self, engine: &mut EngineBorrow) {
        for event in engine.globals.new_messages_of::<WindowEvent<'static>>() {
            // The original winit bindings handle any device event in order to allow for receiving
            // input events that were activated inside the window, and deactivated outside of it.
            //
            // Currently the message bus only sends window events, so uh yeah, that may need to be
            // adjusted.
            //
            // TODO: make Dear ImGui handle all events, not just WindowEvents

            //self.handle_event(io, event);
            self.handle_window_event(event);
        }

        
        

        let imgui = &mut self.imgui;

        // Pre-UI setup
        let io = imgui.io_mut();
        let cursor_change_allowed = !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE);
        io.update_delta_time(
            engine
                .globals
                .get::<FrameHistory>()
                .read()
                .back()
                .map(FrameTiming::controller_time)
                .unwrap_or(Duration::from_secs_f64(1.0 / 60.0)),
        );
        if io.want_set_mouse_pos {
            let position = LogicalPosition::new(io.mouse_pos[0], io.mouse_pos[1]);
            if let Err(e) = self.window.set_cursor_position(position) {
                error!("imgui requested to set cursor position, but the operation failed");
                debug!("error info: {e:#?}");
            }
        }

        let mut textures = DevUiTextures {
            accumulator: &mut self.texture_id_accumulator,
            new_textures: vec![],
            textures_to_remove: vec![],
        };

        if self.font_needs_update {
            self.font_needs_update = false;

            // Wish we could've used alpha8 here, but texture sampling would return zeroes for
            // green and blue channels during fetches, and I don't feel like changing the
            // shader code for that
            let atlas = imgui.fonts().build_rgba32_texture();

            let texture = engine.renderer.create_texture(&TextureDescriptor {
                name: String::from("imgui font atlas"),
                size: uvec2(atlas.width, atlas.height),
                mip_levels: 1,
                format: wgpu::TextureFormat::Rgba8Unorm,
                unfiltered: false,
            });

            engine.renderer.write_texture(&TextureWriteDescriptor {
                handle: &texture,
                mip_level: 0,
                data: atlas.data,
            });

            textures
                .new_textures
                .push((TextureId::new(0), ImguiTexture::from_texture(texture)))
        }

        // UI rendering
        let ui = imgui.new_frame();

        ui.show_demo_window(&mut true);

        // To allow widgets to get `&mut EngineBorrow` we need to not hold a borrow on the universe
        // So, we just collect it to a vector
        let widgets = engine
            .universe
            .get_components_mut::<DevUiComponent>()
            .filter_map(|(entity, component)| Some((entity, component.widget.take()?)))
            .collect::<Vec<_>>();

        for (entity, mut widget) in widgets {
            let keep = widget.process_ui(ui, engine, &mut textures);

            if keep {
                engine
                    .universe
                    .get_component_mut::<DevUiComponent>(entity)
                    .unwrap()
                    .widget = Some(widget);
            } else {
                engine.universe.delete_entity(entity);
            }
        }

        // Post-frame setup
        if cursor_change_allowed {
            let cursor = ui.mouse_cursor();

            if self.current_cursor_cache != cursor {
                self.current_cursor_cache = cursor;

                match cursor {
                    Some(mouse_cursor) => {
                        self.window.set_cursor_visible(true);
                        self.window.set_cursor_icon(to_winit_cursor(mouse_cursor));
                    }
                    _ => self.window.set_cursor_visible(false),
                }
            }
        }

        engine.renderer.imgui_frame = Some(ImguiFrame {
            draw_data: ImguiRenderData::new(imgui.render()),
            new_textures: textures.new_textures,
            textures_to_remove: textures.textures_to_remove,
        });
    }

    #[allow(dead_code)]
    fn handle_event<T>(&mut self, event: &Event<T>) {
        let io = self.imgui.io_mut();

        match *event {
            Event::WindowEvent {
                window_id,
                ref event,
            } => {
                if window_id == self.window.id() {
                    self.handle_window_event(event);
                }
            }

            // Track key release events outside our window. If we don't do this,
            // we might never see the release event if some other window gets focus.
            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(key),
                        ..
                    }),
                ..
            } => {
                if let Some(key) = to_imgui_key(key) {
                    io.add_key_event(key, false);
                }
            }

            _ => {}
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        let window = &self.window;
        let io = self.imgui.io_mut();

        match *event {
            WindowEvent::Resized(physical_size) => {
                let logical_size = physical_size.to_logical::<f32>(window.scale_factor());
                io.display_size = [logical_size.width, logical_size.height];
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                // We need to track modifiers separately because some system like macOS, will
                // not reliably send modifier states during certain events like ScreenCapture.
                // Gotta let the people show off their pretty imgui widgets!
                io.add_key_event(Key::ModShift, modifiers.shift());
                io.add_key_event(Key::ModCtrl, modifiers.ctrl());
                io.add_key_event(Key::ModAlt, modifiers.alt());
                io.add_key_event(Key::ModSuper, modifiers.logo());
            }

            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;

                // We map both left and right ctrl to `ModCtrl`, etc.
                // imgui is told both "left control is pressed" and
                // "consider the control key is pressed". Allows
                // applications to use either general "ctrl" or a
                // specific key. Same applies to other modifiers.
                // https://github.com/ocornut/imgui/issues/5047
                if key == VirtualKeyCode::LShift || key == VirtualKeyCode::RShift {
                    io.add_key_event(imgui::Key::ModShift, pressed);
                } else if key == VirtualKeyCode::LControl || key == VirtualKeyCode::RControl {
                    io.add_key_event(imgui::Key::ModCtrl, pressed);
                } else if key == VirtualKeyCode::LAlt || key == VirtualKeyCode::RAlt {
                    io.add_key_event(imgui::Key::ModAlt, pressed);
                } else if key == VirtualKeyCode::LWin || key == VirtualKeyCode::RWin {
                    io.add_key_event(imgui::Key::ModSuper, pressed);
                }

                // Add main key event
                if let Some(key) = to_imgui_key(key) {
                    io.add_key_event(key, pressed);
                }
            }

            WindowEvent::ReceivedCharacter(ch) => {
                // Exclude the backspace key ('\u{7f}'). Otherwise we will insert this char and then
                // delete it.
                if ch != '\u{7f}' {
                    io.add_input_character(ch)
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let position = position.to_logical::<f32>(window.scale_factor());
                io.add_mouse_pos_event([position.x, position.y]);
            }

            WindowEvent::MouseWheel {
                delta,
                phase: TouchPhase::Moved,
                ..
            } => {
                let (h, v) = match delta {
                    MouseScrollDelta::LineDelta(h, v) => (h, v),
                    MouseScrollDelta::PixelDelta(pos) => {
                        let pos = pos.to_logical::<f64>(window.scale_factor());
                        let h = match pos.x.partial_cmp(&0.0) {
                            Some(Ordering::Greater) => 1.0,
                            Some(Ordering::Less) => -1.0,
                            _ => 0.0,
                        };
                        let v = match pos.y.partial_cmp(&0.0) {
                            Some(Ordering::Greater) => 1.0,
                            Some(Ordering::Less) => -1.0,
                            _ => 0.0,
                        };
                        (h, v)
                    }
                };
                io.add_mouse_wheel_event([h, v]);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(mb) = to_imgui_mouse_button(button) {
                    let pressed = state == ElementState::Pressed;
                    io.add_mouse_button_event(mb, pressed);
                }
            }

            WindowEvent::Focused(newly_focused) => {
                if !newly_focused {
                    // Set focus-lost to avoid stuck keys (like 'alt' when alt-tabbing)
                    io.app_focus_lost = true;
                }
            }
            _ => (),
        }
    }
}

pub struct DevUiTextures<'a> {
    accumulator: &'a mut usize,
    new_textures: Vec<(TextureId, ImguiTexture)>,
    textures_to_remove: Vec<TextureId>,
}

impl<'a> DevUiTextures<'a> {
    pub fn add_texture(&mut self, texture: ImguiTexture) -> TextureId {
        let id = self.next_texture_id();
        self.new_textures.push((id, texture));
        id
    }

    pub fn remove_texture(&mut self, id: TextureId) {
        self.textures_to_remove.push(id);
    }

    fn next_texture_id(&mut self) -> imgui::TextureId {
        let id = *self.accumulator;
        *self.accumulator = id
            .checked_add(1)
            .expect("somehow the texture ID accumulator overflowed");
        imgui::TextureId::new(id)
    }
}

pub struct DevUiComponent {
    /// The widget is taken out during processing
    pub widget: Option<Box<dyn DevUiWidget>>,
}

impl Component for DevUiComponent {}

pub trait DevUiWidget: Send + Sync {
    fn process_ui(
        &mut self,
        ui: &mut imgui::Ui,
        engine: &mut EngineBorrow,
        textures: &mut DevUiTextures,
    ) -> bool;
}

impl<F> DevUiWidget for F
where
    F: FnMut(&mut imgui::Ui, &mut EngineBorrow, &mut DevUiTextures) -> bool + Send + Sync,
{
    fn process_ui(
        &mut self,
        ui: &mut imgui::Ui,
        engine: &mut EngineBorrow,
        textures: &mut DevUiTextures,
    ) -> bool {
        (self)(ui, engine, textures)
    }
}

fn to_winit_cursor(cursor: imgui::MouseCursor) -> MouseCursor {
    match cursor {
        imgui::MouseCursor::Arrow => MouseCursor::Default,
        imgui::MouseCursor::TextInput => MouseCursor::Text,
        imgui::MouseCursor::ResizeAll => MouseCursor::Move,
        imgui::MouseCursor::ResizeNS => MouseCursor::NsResize,
        imgui::MouseCursor::ResizeEW => MouseCursor::EwResize,
        imgui::MouseCursor::ResizeNESW => MouseCursor::NeswResize,
        imgui::MouseCursor::ResizeNWSE => MouseCursor::NwseResize,
        imgui::MouseCursor::Hand => MouseCursor::Hand,
        imgui::MouseCursor::NotAllowed => MouseCursor::NotAllowed,
    }
}

fn to_imgui_mouse_button(button: MouseButton) -> Option<imgui::MouseButton> {
    match button {
        MouseButton::Left | MouseButton::Other(0) => Some(imgui::MouseButton::Left),
        MouseButton::Right | MouseButton::Other(1) => Some(imgui::MouseButton::Right),
        MouseButton::Middle | MouseButton::Other(2) => Some(imgui::MouseButton::Middle),
        MouseButton::Other(3) => Some(imgui::MouseButton::Extra1),
        MouseButton::Other(4) => Some(imgui::MouseButton::Extra2),
        _ => None,
    }
}

fn to_imgui_key(keycode: VirtualKeyCode) -> Option<Key> {
    match keycode {
        VirtualKeyCode::Tab => Some(Key::Tab),
        VirtualKeyCode::Left => Some(Key::LeftArrow),
        VirtualKeyCode::Right => Some(Key::RightArrow),
        VirtualKeyCode::Up => Some(Key::UpArrow),
        VirtualKeyCode::Down => Some(Key::DownArrow),
        VirtualKeyCode::PageUp => Some(Key::PageUp),
        VirtualKeyCode::PageDown => Some(Key::PageDown),
        VirtualKeyCode::Home => Some(Key::Home),
        VirtualKeyCode::End => Some(Key::End),
        VirtualKeyCode::Insert => Some(Key::Insert),
        VirtualKeyCode::Delete => Some(Key::Delete),
        VirtualKeyCode::Back => Some(Key::Backspace),
        VirtualKeyCode::Space => Some(Key::Space),
        VirtualKeyCode::Return => Some(Key::Enter),
        VirtualKeyCode::Escape => Some(Key::Escape),
        VirtualKeyCode::LControl => Some(Key::LeftCtrl),
        VirtualKeyCode::LShift => Some(Key::LeftShift),
        VirtualKeyCode::LAlt => Some(Key::LeftAlt),
        VirtualKeyCode::LWin => Some(Key::LeftSuper),
        VirtualKeyCode::RControl => Some(Key::RightCtrl),
        VirtualKeyCode::RShift => Some(Key::RightShift),
        VirtualKeyCode::RAlt => Some(Key::RightAlt),
        VirtualKeyCode::RWin => Some(Key::RightSuper),
        //VirtualKeyCode::Menu => Some(Key::Menu), // TODO: find out if there is a Menu key in winit
        VirtualKeyCode::Key0 => Some(Key::Alpha0),
        VirtualKeyCode::Key1 => Some(Key::Alpha1),
        VirtualKeyCode::Key2 => Some(Key::Alpha2),
        VirtualKeyCode::Key3 => Some(Key::Alpha3),
        VirtualKeyCode::Key4 => Some(Key::Alpha4),
        VirtualKeyCode::Key5 => Some(Key::Alpha5),
        VirtualKeyCode::Key6 => Some(Key::Alpha6),
        VirtualKeyCode::Key7 => Some(Key::Alpha7),
        VirtualKeyCode::Key8 => Some(Key::Alpha8),
        VirtualKeyCode::Key9 => Some(Key::Alpha9),
        VirtualKeyCode::A => Some(Key::A),
        VirtualKeyCode::B => Some(Key::B),
        VirtualKeyCode::C => Some(Key::C),
        VirtualKeyCode::D => Some(Key::D),
        VirtualKeyCode::E => Some(Key::E),
        VirtualKeyCode::F => Some(Key::F),
        VirtualKeyCode::G => Some(Key::G),
        VirtualKeyCode::H => Some(Key::H),
        VirtualKeyCode::I => Some(Key::I),
        VirtualKeyCode::J => Some(Key::J),
        VirtualKeyCode::K => Some(Key::K),
        VirtualKeyCode::L => Some(Key::L),
        VirtualKeyCode::M => Some(Key::M),
        VirtualKeyCode::N => Some(Key::N),
        VirtualKeyCode::O => Some(Key::O),
        VirtualKeyCode::P => Some(Key::P),
        VirtualKeyCode::Q => Some(Key::Q),
        VirtualKeyCode::R => Some(Key::R),
        VirtualKeyCode::S => Some(Key::S),
        VirtualKeyCode::T => Some(Key::T),
        VirtualKeyCode::U => Some(Key::U),
        VirtualKeyCode::V => Some(Key::V),
        VirtualKeyCode::W => Some(Key::W),
        VirtualKeyCode::X => Some(Key::X),
        VirtualKeyCode::Y => Some(Key::Y),
        VirtualKeyCode::Z => Some(Key::Z),
        VirtualKeyCode::F1 => Some(Key::F1),
        VirtualKeyCode::F2 => Some(Key::F2),
        VirtualKeyCode::F3 => Some(Key::F3),
        VirtualKeyCode::F4 => Some(Key::F4),
        VirtualKeyCode::F5 => Some(Key::F5),
        VirtualKeyCode::F6 => Some(Key::F6),
        VirtualKeyCode::F7 => Some(Key::F7),
        VirtualKeyCode::F8 => Some(Key::F8),
        VirtualKeyCode::F9 => Some(Key::F9),
        VirtualKeyCode::F10 => Some(Key::F10),
        VirtualKeyCode::F11 => Some(Key::F11),
        VirtualKeyCode::F12 => Some(Key::F12),
        VirtualKeyCode::Apostrophe => Some(Key::Apostrophe),
        VirtualKeyCode::Comma => Some(Key::Comma),
        VirtualKeyCode::Minus => Some(Key::Minus),
        VirtualKeyCode::Period => Some(Key::Period),
        VirtualKeyCode::Slash => Some(Key::Slash),
        VirtualKeyCode::Semicolon => Some(Key::Semicolon),
        VirtualKeyCode::Equals => Some(Key::Equal),
        VirtualKeyCode::LBracket => Some(Key::LeftBracket),
        VirtualKeyCode::Backslash => Some(Key::Backslash),
        VirtualKeyCode::RBracket => Some(Key::RightBracket),
        VirtualKeyCode::Grave => Some(Key::GraveAccent),
        VirtualKeyCode::Capital => Some(Key::CapsLock),
        VirtualKeyCode::Scroll => Some(Key::ScrollLock),
        VirtualKeyCode::Numlock => Some(Key::NumLock),
        VirtualKeyCode::Snapshot => Some(Key::PrintScreen),
        VirtualKeyCode::Pause => Some(Key::Pause),
        VirtualKeyCode::Numpad0 => Some(Key::Keypad0),
        VirtualKeyCode::Numpad1 => Some(Key::Keypad1),
        VirtualKeyCode::Numpad2 => Some(Key::Keypad2),
        VirtualKeyCode::Numpad3 => Some(Key::Keypad3),
        VirtualKeyCode::Numpad4 => Some(Key::Keypad4),
        VirtualKeyCode::Numpad5 => Some(Key::Keypad5),
        VirtualKeyCode::Numpad6 => Some(Key::Keypad6),
        VirtualKeyCode::Numpad7 => Some(Key::Keypad7),
        VirtualKeyCode::Numpad8 => Some(Key::Keypad8),
        VirtualKeyCode::Numpad9 => Some(Key::Keypad9),
        VirtualKeyCode::NumpadDecimal => Some(Key::KeypadDecimal),
        VirtualKeyCode::NumpadDivide => Some(Key::KeypadDivide),
        VirtualKeyCode::NumpadMultiply => Some(Key::KeypadMultiply),
        VirtualKeyCode::NumpadSubtract => Some(Key::KeypadSubtract),
        VirtualKeyCode::NumpadAdd => Some(Key::KeypadAdd),
        VirtualKeyCode::NumpadEnter => Some(Key::KeypadEnter),
        VirtualKeyCode::NumpadEquals => Some(Key::KeypadEqual),
        _ => None,
    }
}
