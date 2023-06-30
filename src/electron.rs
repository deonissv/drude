use crate::cfg::ELEC_ELEC_RADIUS;
use crate::collidable::Collidable;

use crate::collidables::Collidables;
use crate::utils::calc_time_to_collision;

extern crate nalgebra as na;
use na::Vector2;

#[derive(Clone)]
pub struct Electron {
    pub pos: Vector2<f64>,
    pub vel: Vector2<f64>,
    pub acc: Vector2<f64>,
    pub time_to_bounce: f64,
    pub collidable: Collidables,
    ticks_since_bounce: f64,
    pub avg_ticks_between_bounces: f64,
    bounce_count: i32,
}

impl Electron {
    pub fn new(pos: Vector2<f64>, vel: Vector2<f64>, acc: Vector2<f64>) -> Electron {
        Electron {
            pos,
            vel,
            acc,
            time_to_bounce: 0.0,
            collidable: Collidables::empty(),
            ticks_since_bounce: 0.0,
            avg_ticks_between_bounces: 0.0,
            bounce_count: 0,
        }
    }

    pub fn update(&mut self, time: f64, supp: f64) {
        let acc = self.acc - self.vel * supp;
        let vel = self.vel;
        self.vel += acc * time;
        self.pos += vel * time + acc * time.powi(2) / 2.0;
        self.time_to_bounce -= time;
        self.ticks_since_bounce += time;
    }

    pub fn update_stats(&mut self) {
        let bounces_time = self.avg_ticks_between_bounces * self.bounce_count as f64;
        self.bounce_count += 1;
        self.avg_ticks_between_bounces =
            (bounces_time + self.ticks_since_bounce) / self.bounce_count as f64;
        self.ticks_since_bounce = 0.0;
    }
}

impl Collidable for Electron {
    fn calc_time_to_collision(&self, other: &Electron) -> f64 {
        calc_time_to_collision(
            self.pos,
            self.vel,
            self.acc,
            other.pos,
            other.vel,
            other.acc,
            ELEC_ELEC_RADIUS,
        )
    }

    fn bounce(&mut self, other: &mut Electron) {
        let dist_vec = other.pos - self.pos;
        if dist_vec.x == 0.0 {
            std::mem::swap(&mut self.vel.y, &mut other.vel.y);
            return;
        }

        let k = dist_vec.y / dist_vec.x;

        let v1_x = self.vel.x - other.vel.x;
        let v1_y = self.vel.y - other.vel.y;

        let v2_x = k * (k * v1_x - v1_y) / (k.powi(2) + 1.0);
        let v2_y = (v1_y - k * v1_x) / (k.powi(2) + 1.0);

        let reflected1 = Vector2::new(v2_x, v2_y) + other.vel;
        let reflected2 = self.vel + other.vel - reflected1;

        self.vel = reflected1;
        other.vel = reflected2;
    }
}

impl PartialEq for Electron {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.vel == other.vel
    }
}

impl Eq for Electron {}

impl Ord for Electron {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.pos.x < other.pos.x {
            std::cmp::Ordering::Less
        } else if self.pos.x > other.pos.x {
            return std::cmp::Ordering::Greater;
        } else if self.pos.y < other.pos.y {
            return std::cmp::Ordering::Less;
        } else if self.pos.y > other.pos.y {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Equal;
        }
    }
}

impl PartialOrd for Electron {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounce_swap_velocity_horiz_left() {
        let mut e1 = Electron::new(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let mut e2 = Electron::new(
            Vector2::new(6.5, 0.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
        );

        e1.bounce(&mut e2);

        assert_eq!(e1.vel, Vector2::new(0.0, 0.0));
        assert_eq!(e2.vel, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn bounce_swap_velocity_horiz_both() {
        let mut e1 = Electron::new(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let mut e2 = Electron::new(
            Vector2::new(6.5, 0.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );

        e1.bounce(&mut e2);

        assert_eq!(e1.vel, Vector2::new(2.0, 0.0));
        assert_eq!(e2.vel, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn bounce_swap_velocity_vert_top() {
        let mut e1 = Electron::new(
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        let mut e2 = Electron::new(
            Vector2::new(0.0, 6.5),
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
        );

        e1.bounce(&mut e2);

        assert_eq!(e1.vel, Vector2::new(0.0, 0.0));
        assert_eq!(e2.vel, Vector2::new(0.0, 2.0));
    }

    #[test]
    fn bounce_swap_velocity_vert_both() {
        let mut e1 = Electron::new(
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        let mut e2 = Electron::new(
            Vector2::new(0.0, 6.5),
            Vector2::new(0.0, 1.0),
            Vector2::new(0.0, 0.0),
        );

        e1.bounce(&mut e2);

        assert_eq!(e1.vel, Vector2::new(0.0, 1.0));
        assert_eq!(e2.vel, Vector2::new(0.0, 2.0));
    }
}
