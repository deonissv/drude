// use std::collections::HashSet;

// use crate::collidable::Collidable;
// use crate::collidables::Collidables;
// use crate::{border::Border, electron::Electron, ion::Ion};

// #[derive(Clone)]
// pub struct Collision(pub (Collidables, Electron));

// impl Collision {
//     pub fn new(col1: Collidables, col2: Electron) -> Collision {
//         Collision((col1, col2))
//     }

//     pub fn calc_time_to_collision(&self) -> f64 {
//         return self.0 .0.calc_time_to_collision(&self.0 .1);
//     }
// }

// impl std::hash::Hash for Collision {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.hash(state);
//     }
// }

// impl std::cmp::PartialEq for Collision {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 .1 == other.0 .1 && self.0 .0 == other.0 .0
//     }
// }

// impl std::cmp::Eq for Collision {}
