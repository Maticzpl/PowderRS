use crate::types::*;

use crate::types::PartBehaviour::Fluid;

pub const PT_WATR : PartType = PartType {
    id: 3,
    name: "WATR",
    col: [0,0,255],
    behaviour: Fluid,
    density: 5,
    graphics: no_gfx,
    update: no_update,
};

// pub fn gfx(sim : &Simulation, pt : &Particle) -> Color{
//     let (x, y) = (pt.x as isize, pt.y as isize);
//
//     let rangeX = 10isize;
//     let rangeY = 10isize;
//     let scale = 2;
//
//     let mut neighbors : f32 = 0f32;
//     for rx in -rangeX..rangeX+1 {
//         for ry in -rangeY..1 {
//             if rx != 0 || ry != 0 {
//                 let (nx, ny) = (x + rx * scale, y + ry * scale);
//                 if nx < 0 || nx >= XRES as isize || ny < 0 || ny >= YRES as isize {
//                     continue;
//                 }
//                 if sim.get_pmap_val(nx as usize, ny as usize) != 0 {
//                     neighbors += 1f32;
//                 }
//             }
//         }
//     }
//     let total = (2 * rangeX * rangeY) as f32;
//     let foam = Color::fade(&Color::SKYBLUE, 1f32 - (neighbors/total));
//     return Color::color_alpha_blend(&pt.get_type().col, &foam, &Color::WHITE);
//
// }