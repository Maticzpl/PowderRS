use std::borrow::{Borrow, BorrowMut};
use raylib::prelude::*;
use crate::types::*;

pub const SCALE : usize = 1;
pub const XRES : usize = 775 / SCALE;
pub const YRES : usize = 575 / SCALE;
pub const XYRES : usize = XRES * YRES;
pub const PT_EMPTY : Particle = Particle{p_type: PT_NONE.id, x: 0, y: 0};


#[derive(Copy, Clone)]
pub struct Particle {
    pub p_type : u16,
    pub x: usize,
    pub y: usize
}
impl Particle {
    pub fn get_type(&self) -> PartType {
        return PT_TYPES[self.p_type as usize];
    }
}

pub struct Simulation {
    pub parts : Vec<Particle>,
    pub pmap : Vec<usize>, //2D array of particle indexes offset by 1 (0 is none)
    partCount : usize,
}
impl Simulation {
    pub fn new() -> Self {
        let mut p = Vec::with_capacity(XYRES);
        p.resize(XYRES, PT_EMPTY);
        let mut pm = Vec::with_capacity(XYRES);
        pm.resize(XYRES, 0);
        Self {
            parts: p,
            pmap: pm,
            partCount : 0
        }
    }

    pub fn add_part(&mut self, part : Particle) -> Result<usize, ()> {
        for i in 0..self.parts.len() {
            if self.parts[i].p_type == 0 {
                self.parts[i] = part;
                self.partCount += 1;
                return Ok(i);
            }
        }
        return Err(());
    }

    pub fn kill_part(&mut self, id : usize) -> Result<(),()> {
        if id >= self.parts.len() || self.parts[id].p_type == 0 {
            return Err(());
        }
        self.parts[id].p_type = PT_NONE.id;
        self.partCount -= 1;
        return Ok(());
    }

    pub fn get_part(&mut self, id : usize) -> Result<&Particle,()> {
        if id >= self.parts.len() {
            return Err(());
        }
        return Ok(&self.parts[id]);
    }

    pub fn get_pmap(&mut self, x : usize, y : usize) -> Result<&Particle,()> {
        if x >= XRES || y >= YRES {
            return Err(());
        }

        let val = self.pmap[x+(y*XRES)];
        if val == 0 {
            return Err(());
        }

        return Ok(&self.parts[val - 1]);
    }

    pub fn get_pmap_val(&mut self, x : usize, y : usize) -> usize{
        if x >= XRES || y >= YRES {
            return 0;
        }
        return self.pmap[x+(y*XRES)];
    }

    pub fn update_p_map(&mut self) {
        let mut counter = 0;

        self.pmap.fill(0);
        for i in 0..self.parts.len() {
            if counter >= self.partCount {
                break;
            }

            if self.parts[i].p_type != 0 {
                let index = self.parts[i].x + (self.parts[i].y * XRES);
                self.pmap[index as usize] = i;
                counter += 1;
            }
        }
    }

    pub fn step(&mut self) {
        self.update_p_map();
        let mut counter = 0;

        for i in 0..self.parts.len() {
            if counter >= self.partCount {
                break;
            }

            let p_type = self.parts[i].get_type();
            if p_type.id != 0 {
                match (p_type.behaviour) {
                    PartBehaviour::Skip => {},
                    PartBehaviour::Solid => {
                        (p_type.update)(&mut self.parts[i]);
                    },
                    PartBehaviour::Powder => {
                        self.powder_move(i);
                        (p_type.update)(&mut self.parts[i]);
                    },
                    PartBehaviour::Fluid => {
                        self.liquid_move(i);
                        (p_type.update)(&mut self.parts[i]);
                    },
                    PartBehaviour::Gas => {
                        //TODO uhh make tis xd
                        (p_type.update)(&mut self.parts[i]);
                    },
                }

                counter += 1;
            }
        }
    }

    fn powder_move(&mut self, id : usize) {
        let (x, y) = (self.parts[id].x as i32, self.parts[id].y as i32);

        let ran: i32 = get_random_value(0, 1);

        if self.try_move(id, 0, 1) { return }
        if ran == 1 {
            if self.try_move(id, 1, 1) { return }
        } else {
            if self.try_move(id, -1, 1) { return }
        }
    }

    fn liquid_move(&mut self, id : usize) {
        let (x, y) = (self.parts[id].x as i32, self.parts[id].y as i32);

        let ran: i32 = get_random_value(0, 1);

        if self.try_move(id, 0, 1) { return }
        if ran == 1 {
            if self.try_move(id, 1, 1) { return }
            if self.try_move(id, 1, 0) { return }
        } else {
            if self.try_move(id, -1, 1) { return }
            if self.try_move(id, -1, 0) { return }
        }
    }

    fn try_move(&mut self, i : usize, rx : isize, ry : isize) -> bool {
        let (x, y) = (self.parts[i].x as isize, self.parts[i].y as isize);
        let (nx, ny) = (x + rx, y + ry);
        if nx < 0 || nx >= XRES as isize || ny < 0 || ny >= YRES as isize {
            //self.parts[i].p_type = 0;
            return false;
        }
        let occupying = self.get_pmap_val(nx as usize, ny as usize);
        if occupying != 0 {
            if self.parts[occupying].get_type().density < self.parts[i].get_type().density {    //SWAP
                let (ox, oy) = (self.parts[occupying].x ,self.parts[occupying].y);
                self.parts[occupying].x = self.parts[i].x;
                self.parts[occupying].y = self.parts[i].y;
                self.parts[i].x = ox;
                self.parts[i].y = oy;

                return true;
            }
            return false;
        }
        self.parts[i].x = nx as usize;
        self.parts[i].y = ny as usize;
        return true;
    }
}