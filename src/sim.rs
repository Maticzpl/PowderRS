use crate::types::*;
use rand::prelude::*;

pub const SCALE : usize = 1;
pub const WINW: usize = 1024;
pub const WINH: usize = 576;
pub const UI_MARGIN: usize = 30;
pub const XRES : usize = (WINW - UI_MARGIN) / SCALE;
pub const YRES : usize = (WINH - UI_MARGIN) / SCALE;
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
    pub pmap : Vec<usize>, //2D array of particle indexes offset by 1 (0 is none)
    part_count: usize,
}
impl Simulation {
    #[feature(box_syntax)]
    pub fn new() -> Self {
        let mut p : Box<[Particle; XYRES]> = box[PT_EMPTY; XYRES];
        let mut pm = Vec::with_capacity(XYRES);
        pm.resize(XYRES, 0);

        Self {
            parts: p,
            pmap: pm,
            part_count: 0
        }
    }

    pub fn add_part(&mut self, part : Particle) -> Result<usize, ()> {
        if part.p_type == 0 {
            return Err(());
        }
        if self.get_pmap_val(part.x as usize, part.y as usize) != 0 {
            return Err(());
        }

        for i in 0..self.parts.len() {
            if self.parts[i].p_type == 0 {
                self.parts[i] = part;
                self.part_count += 1;
                return Ok(i);
            }
        }
        return Err(());
    }

    pub fn kill_part(&mut self, id : usize) -> Result<(),()> {
        if id >= self.parts.len() || self.parts[id].p_type == 0 {
            return Err(());
        }
        self.pmap[self.parts[id].x as usize + (self.parts[id].y as usize * XRES)] = 0;
        self.parts[id] = PT_EMPTY;
        self.part_count -= 1;
        return Ok(());
    }

    pub fn get_part(&self, id : usize) -> Result<&Particle,()> {
        if id >= self.parts.len() {
            return Err(());
        }
        return Ok(&self.parts[id]);
    }

    pub fn get_part_count(&self) -> usize {
        return self.part_count;
    }

    pub fn get_pmap(&self, x : usize, y : usize) -> Result<&Particle,()> {
        if x >= XRES || y >= YRES {
            return Err(());
        }

        let val = self.pmap[x+(y*XRES)];
        if val == 0 {
            return Err(());
        }

        return Ok(&self.parts[val - 1]);
    }

    // not adjusted for 0
    pub fn get_pmap_val(&self, x : usize, y : usize) -> usize{
        if x >= XRES || y >= YRES {
            return 0;
        }

        return self.pmap[x+(y*XRES)];
    }

    pub fn update_p_map(&mut self) {
        let mut counter = 0;

        self.pmap.fill(0);
        for i in 0..self.parts.len() {
            // if counter >= self.partCount {
            //     break;
            // }

            if self.parts[i].p_type != 0 {
                let index = self.parts[i].x + (self.parts[i].y * XRES as u32);
                self.pmap[index as usize] = i + 1;
                counter += 1;
            }
        }
        if counter != self.part_count {
            println!("Counter out of sync: {} expected {}", self.part_count, counter);
        }
    }

    pub fn step(&mut self) {
        self.update_p_map();
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
        let (x, y) = (self.parts[id].x as i32, self.parts[id].y as i32);

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
        let (x, y) = (self.parts[id].x as i32, self.parts[id].y as i32);

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
        let mut occupying = self.get_pmap_val(nx as usize, ny as usize);
        if occupying != 0 {
            occupying -= 1;
            if self.parts[occupying].p_type == 0 {
                print!("s");
            }
            if swap && self.parts[occupying].get_type().density < self.parts[i].get_type().density {    //SWAP
                let (ox, oy) = (self.parts[occupying].x ,self.parts[occupying].y);
                self.pmap[ x as usize +  y as usize * XRES] = occupying + 1;
                self.pmap[ox as usize + oy as usize * XRES] = i + 1;

                self.parts[occupying].x = self.parts[i].x;
                self.parts[occupying].y = self.parts[i].y;
                self.parts[i].x = ox;
                self.parts[i].y = oy;

                return true;
            }
            return false;
        }
        self.pmap[nx as usize + ny as usize * XRES] = i + 1;
        self.pmap[ x as usize +  y as usize * XRES] = 0;

        self.parts[i].x = nx as u32;
        self.parts[i].y = ny as u32;
        return true;
    }

    fn check_pmap(&mut self, i : usize, rx : isize, ry : isize) -> usize {
        let (x, y) = (self.parts[i].x as isize, self.parts[i].y as isize);
        let (nx, ny) = (x + rx, y + ry);
        if nx < 0 || nx >= XRES as isize || ny < 0 || ny >= YRES as isize {
            return 0;
        }
        return self.get_pmap_val(nx as usize, ny as usize);
    }
}