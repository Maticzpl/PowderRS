#![feature(box_syntax)]
#![feature(is_some_and)]

mod sim;
mod types;
mod gl_renderer;

use std::collections::{HashMap, HashSet};
use std::iter::Map;
use std::ops::Index;
use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use glium::glutin::event::ElementState::Pressed;
use glium::glutin::platform::unix::x11::ffi::KeyCode;
use rand::{thread_rng};
use crate::gl_renderer::GLRenderer;
use crate::types::*;
use crate::sim::{Particle, Simulation};

fn tick(sim: &mut Simulation, ren: &mut GLRenderer, input: &InputData) {
    sim.step();


    if input.mouse_buttons.get(&MouseButton::Left).is_some_and(|b| *b) {
        let (x, y) = (input.mouse_pos.x as u32, input.mouse_pos.y as u32);

        for i in 0..25 {
            sim.add_part(Particle{p_type:3, x: x + i%5, y: y + i/5});
        }
    }
    if input.mouse_buttons.get(&MouseButton::Right).is_some_and(|b| *b) {
        let (x, y) = (input.mouse_pos.x as u32, input.mouse_pos.y as u32);

        for i in 0..25 {
            sim.kill_part(sim.get_pmap_val((x + i % 5) as usize, (y + i / 5) as usize) - 1);
        }
    }

    ren.draw(&sim);
}

fn main() {
    let mut sim = Simulation::new();
    let mut ren = GLRenderer::new(&sim);
    let mut event_loop = ren.1;
    let mut ren = ren.0;

    let mut input: InputData = InputData {
        mouse_buttons: HashMap::new(),
        keys: HashMap::new(),
        mouse_pos: PhysicalPosition{ x:0.0, y:0.0 },
        scroll: MouseScrollDelta::LineDelta(0.0,0.0)
    };

    for i in 0..100 {
        sim.add_part(Particle{p_type:1, x: i+20, y: i+50});
        sim.add_part(Particle{p_type:1, x: i+120, y: i+50});

        sim.add_part(Particle{p_type:1, x: i * 4, y: 450});
        sim.add_part(Particle{p_type:1, x: (i * 4)+1, y: 450});
        sim.add_part(Particle{p_type:1, x: (i * 4)+2, y: 450});
        sim.add_part(Particle{p_type:1, x: (i * 4)+3, y: 450});

        sim.add_part(Particle{p_type:1, x: 0, y: 450-i});
        sim.add_part(Particle{p_type:1, x: 400, y: 450-i});
    }

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
                        button: button,
                        state: state,
                        ..
                    } => {
                        if state == Pressed {
                            input.mouse_buttons.insert(button, true);
                        } else {
                            input.mouse_buttons.insert(button, false);
                        }
                    }
                    WindowEvent::MouseWheel {
                        delta: delta,
                        ..
                    } => {
                        input.scroll = delta;
                    }
                    WindowEvent::CursorMoved {
                        position: pos,
                        ..
                    } => {
                        input.mouse_pos = pos;
                    }
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: key,
                            state: state,
                            scancode: scan,
                            ..
                        },
                        ..
                    } => {
                        let key = key.unwrap();
                        if state == Pressed {
                            input.keys.insert(key, true);
                        } else {
                            input.keys.insert(key, false);
                        }
                    },

                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                tick(&mut sim, &mut ren, &input);
            },
            _ => {}

        }
    });

}

pub struct InputData {
    pub mouse_buttons: HashMap<MouseButton, bool>,
    pub keys: HashMap<VirtualKeyCode, bool>,
    pub mouse_pos: PhysicalPosition<f64>,
    pub scroll: MouseScrollDelta
}

