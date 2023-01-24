#![feature(box_syntax)]
#![feature(is_some_and)]

mod sim;
mod types;
mod gl_renderer;
mod gui;

use std::collections::{HashMap, HashSet};
use std::iter::Map;
use std::ops::Index;
use cgmath::num_traits::pow;
use cgmath::{Matrix3, Matrix4, MetricSpace, Transform, Vector2, Vector3, Vector4, VectorSpace, Zero};
use glium::glutin::dpi::{PhysicalPosition, PhysicalSize};
use glium::glutin::event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use glium::glutin::event::ElementState::Pressed;
use glium::glutin::event::MouseScrollDelta::LineDelta;
use glium::glutin::platform::unix::x11::ffi::KeyCode;
use rand::{thread_rng};
use crate::gl_renderer::GLRenderer;
use crate::types::*;
use crate::sim::{Particle, Simulation, WINH, WINW, XRES, YRES};

struct TickFnState {
    pan_started: bool,
    pan_start_pos: Vector2<f32>,
    pan_original: Vector2<f32>,
    brush_size: i32,
}

fn tick(sim: &mut Simulation, ren: &mut GLRenderer, input: &mut InputData, tick_state : &mut TickFnState) {
    sim.step();

    // Correct mouse pos
    let mut mouse_pos = Vector4 {x: input.mouse_pos.x as f32, y: input.mouse_pos.y as f32, z: 0.0, w : 1.0};
    let (mut sx, mut sy) = (input.win_size.width as f32 / WINW as f32, input.win_size.height as f32 / WINH as f32);
    let mouse_screen_pos = Vector4 {x : mouse_pos.x / sx, y: mouse_pos.y / sy, z: 0.0, w: 1.0};
    mouse_pos =
        Matrix4::from_translation(Vector3 {x:  (WINW as f32/2.0), y:  (WINH as f32/2.0), z: 0.0}) *
        ren.view_matrix.inverse_transform().expect("") *
        Matrix4::from_translation(Vector3 {x: -(WINW as f32/2.0), y: -(WINH as f32/2.0), z: 0.0}) *
        mouse_screen_pos;


    if input.mouse_buttons.get(&MouseButton::Left).is_some_and(|b| *b) {
        let size = tick_state.brush_size as usize;
        let hs = size as usize / 2usize;
        let (mut x, mut y) = (mouse_pos.x as usize, mouse_pos.y as usize);
        x = x.clamp(hs, (XRES - hs - 1) as usize);
        y = y.clamp(hs, (YRES - hs - 1) as usize);

        for i in 0..pow(size, 2) {
            sim.add_part(Particle { p_type: 2, x: (x - hs + i / size) as u32, y: (y - hs + i % size) as u32 });
        }
    }
    if input.mouse_buttons.get(&MouseButton::Right).is_some_and(|b| *b) {
        let size = tick_state.brush_size as usize;
        let hs = size as usize / 2usize;
        let (mut x, mut y) = (mouse_pos.x as usize, mouse_pos.y as usize);
        x = x.min((XRES - size) as usize);
        y = y.min((YRES - size) as usize);

        for i in 0..pow(size, 2) {
            let val = sim.get_pmap_val((x + i % size) as usize, (y + i / size) as usize);
            if val != 0 {
                sim.kill_part(val - 1);
            }
        }
    }


    if input.mouse_buttons.get(&MouseButton::Middle).is_some_and(|b| *b) {
        let (x, y) = (mouse_screen_pos.x as f32, mouse_screen_pos.y as f32);
        if !tick_state.pan_started {
            tick_state.pan_start_pos = Vector2 {x, y};
            tick_state.pan_started = true;
            tick_state.pan_original = Vector2::from(ren.camera_pan);
        } else {
            ren.camera_pan.x = (tick_state.pan_original.x + (x - tick_state.pan_start_pos.x) / ren.camera_zoom);
            ren.camera_pan.y = (tick_state.pan_original.y + (y - tick_state.pan_start_pos.y) / ren.camera_zoom);
        }
    } else {
        tick_state.pan_started = false;
    }


    if input.keys.get(&VirtualKeyCode::LControl).is_some_and(|b| *b)
        && input.scroll != 0.0 {

        let change = input.scroll / 10.0 * (ren.camera_zoom*2.0);
        let prev_zoom = ren.camera_zoom.clone();
        ren.camera_zoom += change;
        ren.camera_zoom = ren.camera_zoom.clamp(1.0, 15.0);
        let res =
            Matrix4::from_translation( Vector3::from([(WINW/2) as f32, (WINH/2) as f32, 0.0])) *
            Matrix4::from_translation(-Vector3{x:ren.camera_pan.x, y:ren.camera_pan.y, z:0.0}) *
            Matrix4::from_scale(prev_zoom/ren.camera_zoom) *
            Matrix4::from_translation(Vector3{x:ren.camera_pan.x, y:ren.camera_pan.y, z:0.0}) *
            Matrix4::from_translation(-Vector3::from([(WINW/2) as f32, (WINH/2) as f32, 0.0])) *
            mouse_pos;

        ren.camera_pan += (res - mouse_pos).truncate().truncate();

        input.scroll = 0.0;
    } else if input.scroll != 0.0 {
        tick_state.brush_size += input.scroll.signum() as i32;
        tick_state.brush_size = tick_state.brush_size.clamp(1 ,20);
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
    sim.add_part(Particle{p_type:1, x: (WINW / 2) as u32, y: (WINH / 2) as u32 });

    let mut tick_state = TickFnState {
        pan_started: false,
        pan_start_pos: Vector2 {x: 0.0, y: 0.0},
        pan_original: Vector2::from([0.0, 0.0]),
        brush_size: 5,
    };

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

                        if key.is_some() {
                            let key = key.unwrap();
                            if state == Pressed {
                                input.keys.insert(key, true);
                            } else {
                                input.keys.insert(key, false);
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

