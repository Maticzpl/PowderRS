use raylib::prelude::*;
use crate::sim;
use crate::sim::Simulation;

pub trait Renderer{
    fn draw(&self, sim : &Simulation);
    fn should_close(&self)-> bool;
    fn get_window_size(&self) -> (i32, i32);
}