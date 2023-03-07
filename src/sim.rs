use crate::types::*;
use rand::prelude::*;

pub const WINW: usize = (1.3 * 1024.0) as usize;
pub const WINH: usize = (1.3 * 576.0) as usize;
pub const UI_MARGIN: usize = 30;
pub const XRES : usize = WINW - UI_MARGIN;
pub const YRES : usize = WINH - UI_MARGIN;
pub const XYRES : usize = XRES * YRES;
pub const PT_EMPTY : Particle = Particle{p_type: PT_NONE.id, x: 0, y: 0};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Particle {
    pub p_type : u32,
    pub x: u32,
    pub y: u32
}
impl Particle {
    pub fn get_type(&self) -> PartType {
        return PT_TYPES[self.p_type as usize];
    }
}

pub struct Simulation {
    pub parts : Box<[Particle; XYRES]>,
    pub pmap : Box<[Option<usize>; XYRES]>,
    part_count: usize,
}
impl Simulation {
    pub fn new() -> Self {
        let p: Box<[Particle; XYRES]> = box[PT_EMPTY; XYRES];
        let pm: Box<[Option<usize>; XYRES]> = box[None; XYRES];

        Self {
            parts: p,
            pmap: pm,
            part_count: 0,
        }
    }

    pub fn add_part(&mut self, part : Particle) -> Option<usize> {
        if part.p_type == 0 {
            return None;
        }
        if self.get_pmap_val(part.x as usize, part.y as usize).is_some() {
            return None;
        }

        for i in 0..self.parts.len() {
            if self.parts[i].p_type == 0 {
                self.parts[i] = part;
                self.part_count += 1;
                self.pmap[part.x as usize + (part.y as usize * XRES)] = Some(i);
                return Some(i);
            }
        }
        return None;
    }

    pub fn kill_part(&mut self, id : usize) -> Result<(),()> {
        if id >= self.parts.len() || self.parts[id].p_type == 0 {
            return Err(());
        }
        self.pmap[self.parts[id].x as usize + (self.parts[id].y as usize * XRES)] = None;
        self.parts[id] = PT_EMPTY;
        self.part_count -= 1;
        return Ok(());
    }

    pub fn get_part(&self, id : usize) -> &Particle {
        return &self.parts[id];
    }

    pub fn get_part_count(&self) -> usize {
        return self.part_count;
    }

    pub fn get_pmap(&self, x : usize, y : usize) -> Option<&Particle> {
        if x >= XRES || y >= YRES {
            return None;
        }

        let val = self.pmap[x+(y*XRES)];
        if val.is_none() {
            return None;
        }
        return Some(&self.parts[val.unwrap()]);
    }

    pub fn get_pmap_val(&self, x : usize, y : usize) -> Option<usize>{
        if x >= XRES || y >= YRES {
            return None;
        }

        return self.pmap[x+(y*XRES)];
    }

    pub fn update_p_map(&mut self) {
        let mut counter = 0;

        self.pmap.fill(None);
        for i in 0..self.parts.len() {
            if counter >= self.part_count {
                break;
            }

            if self.parts[i].p_type != 0 {
                let index = self.parts[i].x + (self.parts[i].y * XRES as u32);
                self.pmap[index as usize] = Some(i);
                counter += 1;
            }
        }
    }

    pub fn step(&mut self) {
        //self.update_p_map();

        let mut counter = 0;

        for i in (0..self.parts.len()).rev() {
            if counter >= self.part_count {
                break;
            }

            let p_type = self.parts[i].get_type();
            if self.parts[i].p_type != 0 {
                match p_type.behaviour {
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
        let (_x, _y) = (self.parts[id].x as i32, self.parts[id].y as i32);

        let ran: bool = thread_rng().gen();

        if self.try_move(id, 0, 1, true) {
            return;
        }
        if ran {
            self.try_move(id, 1, 1, true);
        } else {
            self.try_move(id, -1, 1, true);
        }
    }

    fn liquid_move(&mut self, id : usize) {
        let (_x, _y) = (self.parts[id].x as i32, self.parts[id].y as i32);

        let ran: bool = thread_rng().gen();
        if self.try_move(id, 0, 1, true) {
            return;
        }
        if ran {
            if self.try_move(id, 1, 1, false) {
                return;
            }
        } else {
            if self.try_move(id, -1, 1, false) {
                return;
            }
        }
        if ran {
            self.try_move(id, 1, 0, false);
        } else {
            self.try_move(id, -1, 0, false);
        }
    }

    fn try_move(&mut self, i : usize, rx : isize, ry : isize, swap :bool) -> bool {
        let (x, y) = (self.parts[i].x as isize, self.parts[i].y as isize);
        let (nx, ny) = (x + rx, y + ry);

        if nx < 0 || nx >= XRES as isize || ny < 0 || ny >= YRES as isize {
            self.kill_part(i).expect("Invalid kill");
            return true; // return false if edges collide
        }
        let occupying = self.get_pmap_val(nx as usize, ny as usize);
        if occupying.is_some() {
            let occupying = occupying.unwrap();
            if self.parts[occupying].p_type == 0 {
                print!("s");
            }
            if swap && self.parts[occupying].get_type().density < self.parts[i].get_type().density {    //SWAP
                let (ox, oy) = (self.parts[occupying].x ,self.parts[occupying].y);
                self.pmap[ x as usize +  y as usize * XRES] = Some(occupying);
                self.pmap[ox as usize + oy as usize * XRES] = Some(i);

                self.parts[occupying].x = self.parts[i].x;
                self.parts[occupying].y = self.parts[i].y;
                self.parts[i].x = ox;
                self.parts[i].y = oy;

                return true;
            }
            return false;
        }
        self.pmap[nx as usize + ny as usize * XRES] = Some(i);
        self.pmap[ x as usize +  y as usize * XRES] = None;

        self.parts[i].x = nx as u32;
        self.parts[i].y = ny as u32;
        return true;
    }
}