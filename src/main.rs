#![feature(box_syntax)]
#![feature(is_some_and)]

mod sim;
mod types;
mod gl_renderer;

use std::collections::{HashMap, HashSet};
use std::iter::Map;
use std::ops::Index;
use cgmath::num_traits::pow;
use cgmath::{Matrix4, Transform, Vector2, Vector3, Vector4, Zero};
use glium::glutin::dpi::{PhysicalPosition, PhysicalSize};
use glium::glutin::event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use glium::glutin::event::ElementState::Pressed;
use glium::glutin::event::MouseScrollDelta::LineDelta;
use glium::glutin::platform::unix::x11::ffi::KeyCode;
use rand::{thread_rng};
use crate::gl_renderer::GLRenderer;
use crate::types::*;
use crate::sim::{Particle, Simulation, WINH, WINW};

fn tick(sim: &mut Simulation, ren: &mut GLRenderer, input: &mut InputData) {
    sim.step();

    let mut pos = Vector4 {x: input.mouse_pos.x as f32, y: input.mouse_pos.y as f32, z: 0.0, w : 1.0};
    let (mut sx, mut sy) = (input.win_size.width as f32 / WINW as f32, input.win_size.height as f32 / WINH as f32);
    pos = Vector4 {x : pos.x / sx, y: pos.y / sy, z: 0.0, w: 1.0};
    pos = Matrix4::from_translation(Vector3 {x: ren.camera_pan[0], y: ren.camera_pan[1], z: 0.0}) *
        ren.view_matrix.inverse_transform().expect("") *
        Matrix4::from_translation(Vector3 {x: -ren.camera_pan[0], y: -ren.camera_pan[1], z: 0.0}) *
        pos;



    if input.mouse_buttons.get(&MouseButton::Left).is_some_and(|b| *b) {
        let (x, y) = (pos.x as u32, pos.y as u32);

        for i in 0..25 {
            sim.add_part(Particle{p_type:3, x: x + i%5, y: y + i/5});
        }
    }
    if input.mouse_buttons.get(&MouseButton::Right).is_some_and(|b| *b) {
        let (x, y) = (pos.x as u32, pos.y as u32);

        for i in 0..25 {
            let val = sim.get_pmap_val((x + i % 5) as usize, (y + i / 5) as usize);
            if val != 0 {
                sim.kill_part(val - 1);
            }
        }
    }

    if input.keys.get(&VirtualKeyCode::LControl).is_some_and(|b| *b)
        &&
        input.scroll != 0.0 {
        ren.camera_zoom += input.scroll / 10.0 * (ren.camera_zoom*2.0);
        ren.camera_zoom = ren.camera_zoom.clamp(1.0, 10.0);
        input.scroll = 0.0;
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
        scroll: 0.0,
        win_size: PhysicalSize { width: WINW as u32, height: WINH as u32 }
    };

    for i in 0..100 {
        sim.add_part(Particle{p_type:1, x: i+20, y: i+50});
        sim.add_part(Particle{p_type:1, x: i+20, y: i+80});
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
                        delta: LineDelta(x, y),
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
                            state: state,
                            scancode: scan,
                            ..
                        },
                        ..
                    } => {
                        //println!("{:?} k-s {}",key,scan);

                        let key = key.unwrap();
                        if state == Pressed {
                            input.keys.insert(key, true);
                        } else {
                            input.keys.insert(key, false);
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
                tick(&mut sim, &mut ren, &mut input);
            },
            _ => {}

        }
    });

}

pub struct InputData {
    pub mouse_buttons: HashMap<MouseButton, bool>,
    pub keys: HashMap<VirtualKeyCode, bool>,
    pub mouse_pos: PhysicalPosition<f64>,
    pub scroll: f32,
    pub win_size: PhysicalSize<u32>,
}

