use std::collections::HashMap;
use glium::glutin::dpi::{PhysicalPosition, PhysicalSize};
use glium::glutin::event::{Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use glium::glutin::event::ElementState::Pressed;
use glium::glutin::event::MouseScrollDelta::LineDelta;
use glium::glutin::event_loop::EventLoop;
use crate::gl_renderer::GLRenderer;
use crate::sim::Simulation;
use crate::{tick, TickFnState};


pub struct InputData {
    pub mouse_buttons: HashMap<MouseButton, bool>,
    pub prev_mouse_buttons: HashMap<MouseButton, bool>,
    pub keys: HashMap<VirtualKeyCode, bool>,
    pub prev_keys: HashMap<VirtualKeyCode, bool>,
    pub mouse_pos: PhysicalPosition<f64>,
    pub scroll: f32,
    pub win_size: PhysicalSize<u32>,
}

impl InputData {
    pub fn key_pressed(&self, key: &VirtualKeyCode) -> bool {
        self.keys.get(key).is_some()
    }
    pub fn key_just_pressed(&self, key: &VirtualKeyCode) -> bool {
        self.keys.get(key).is_some() && self.prev_keys.get(key).is_none()
    }
    pub fn mouse_pressed(&self, button: &MouseButton) -> bool {
        self.mouse_buttons.get(button).is_some()
    }
    pub fn mouse_just_pressed(&self, button: &MouseButton) -> bool {
        self.mouse_buttons.get(button).is_some() && self.prev_mouse_buttons.get(button).is_none()
    }
}


pub fn handle_events(event_loop: EventLoop<()>, mut input: InputData, mut sim: Simulation, mut ren: GLRenderer<'static>, mut tick_state: TickFnState) {

    event_loop.run(move |event, _, flow| {
        match event {
            Event::WindowEvent {
                event: ev,
                ..
            } => {
                match ev {
                    WindowEvent::CloseRequested => {
                        flow.set_exit();
                    }
                    WindowEvent::MouseInput {
                        button,
                        state,
                        ..
                    } => {
                        if state == Pressed {
                            input.mouse_buttons.insert(button, true);
                        } else {
                            input.mouse_buttons.remove(&button);
                        }
                    }
                    WindowEvent::MouseWheel {
                        delta: LineDelta(_x, y),
                        ..
                    } => {
                        input.scroll = y;
                    }
                    WindowEvent::CursorMoved {
                        position: pos,
                        ..
                    } => {
                        input.mouse_pos.x = pos.x as f64;
                        input.mouse_pos.y = pos.y as f64;
                    }
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: key,
                            state,
                            scancode: _scan,
                            ..
                        },
                        ..
                    } => {
                        //println!("{:?} k-s {}",key,_scan);

                        if key.is_some() {
                            let key = key.unwrap();
                            if state == Pressed {
                                input.keys.insert(key, true);
                            } else {
                                input.keys.remove(&key);
                            }
                        }
                    },
                    WindowEvent::Resized {
                        0: size
                    } => {
                        input.win_size = size;
                    }

                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                tick(&mut sim, &mut ren, &mut input, &mut tick_state);
                input.prev_keys = input.keys.clone();
                input.prev_mouse_buttons = input.mouse_buttons.clone();
            },
            _ => {}

        }
    });
}
