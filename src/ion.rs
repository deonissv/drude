extern crate nalgebra as na;
use crate::cfg::ION_ELEC_RADIUS;
use crate::collidable::Collidable;
use crate::electron::Electron;
use crate::utils::calc_time_to_collision;
use na::Vector2;

#[derive(Clone, Copy)]
pub struct Ion {
    pub pos: Vector2<f64>,
}

impl Ion {
    pub fn new(pos: Vector2<f64>) -> Ion {
        Ion { pos }
    }
}

impl Collidable for Ion {
    fn calc_time_to_collision(&self, other: &Electron) -> f64 {
        calc_time_to_collision(
            other.pos,
            other.vel,
            other.acc,
            self.pos,
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
            ION_ELEC_RADIUS,
        )
    }

    fn bounce(&mut self, other: &mut Electron) {
        let normal = (other.pos - self.pos).normalize();
        let incidence = other.vel.normalize() * -1.0;
        let dot = incidence.dot(&normal);
        let mut reflected = normal * 2.0 * dot - incidence;
        reflected *= other.vel.magnitude();

        other.vel = reflected;
    }
}

impl PartialEq for Ion {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for Ion {}

impl std::hash::Hash for Ion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pos.x.to_bits().hash(state);
        self.pos.y.to_bits().hash(state);
    }
}

impl Ord for Ion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.pos.x < other.pos.x {
            std::cmp::Ordering::Less
        } else if self.pos.x > other.pos.x {
            std::cmp::Ordering::Greater
        } else if self.pos.y < other.pos.y {
            std::cmp::Ordering::Less
        } else if self.pos.y > other.pos.y {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for Ion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_time_to_collision_left() {
        let ion = Ion::new(Vector2::new(100.0, 100.0));
        let electron = Electron::new(
            Vector2::new(86.0, 100.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let time = ion.calc_time_to_collision(&electron);
        assert_eq!(time, 0.5);
    }

    #[test]
    fn calc_time_to_collision_right() {
        let ion = Ion::new(Vector2::new(100.0, 100.0));
        let electron = Electron::new(
            Vector2::new(114.0, 100.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let time = ion.calc_time_to_collision(&electron);
        assert_eq!(time, 0.5);
    }

    #[test]
    fn calc_time_to_collision_top() {
        let ion = Ion::new(Vector2::new(100.0, 100.0));
        let electron = Electron::new(
            Vector2::new(100.0, 114.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        );
        let time = ion.calc_time_to_collision(&electron);
        assert_eq!(time, 0.5);
    }

    #[test]
    fn calc_time_to_collision_bottom() {
        let ion = Ion::new(Vector2::new(100.0, 100.0));
        let electron = Electron::new(
            Vector2::new(100.0, 86.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        let time = ion.calc_time_to_collision(&electron);
        assert_eq!(time, 0.5);
    }

    #[test]
    fn calc_time_to_collision_bottom_acc() {
        let ion = Ion::new(Vector2::new(100.0, 100.0));
        let electron = Electron::new(
            Vector2::new(100.0, 86.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 1.0),
        );
        let time = ion.calc_time_to_collision(&electron);
        assert_eq!(time, 0.449_489_742_783_177_9);
    }
}
