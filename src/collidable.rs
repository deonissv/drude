use crate::electron::Electron;

pub trait Collidable {
    fn calc_time_to_collision(&self, other: &Electron) -> f64;
    fn bounce(&mut self, other: &mut Electron);
}
