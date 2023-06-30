use nalgebra::Vector2;

use crate::{cfg::ELECTRON_RADIUS, electron::Electron, utils::calc_time_to_border_collision};

#[derive(Clone, Copy)]
pub enum BorderType {
    Inner,
    Outer,
}

#[derive(Clone, Copy)]
pub struct Border {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub border_type: BorderType,
}

impl Border {
    pub fn new(a: f64, b: f64, c: f64, border_type: BorderType) -> Border {
        Border {
            a,
            b,
            c,
            border_type,
        }
    }

    pub fn bounce(&mut self, other: &mut Electron, width: f64) {
        if self.a == 1.0 && self.b == 0.0 {
            match self.border_type {
                BorderType::Inner => other.pos.x -= width,
                BorderType::Outer => other.pos.x += width,
            }
        } else if self.a == 0.0 && self.b == 1.0 {
            other.vel.y *= -1.0;
        } else {
            panic!("Not implemented: Invalid border");
        }
    }

    pub fn calc_time_to_collision(&self, other: &Electron, width: f64) -> f64 {
        let (dist, vel, acc) = if self.a == 1.0 && self.b == 0.0 {
            let pos = other.pos.x;
            match self.border_type {
                BorderType::Inner => (width + ELECTRON_RADIUS - pos, other.vel.x, other.acc.x),
                BorderType::Outer => (pos + ELECTRON_RADIUS - self.c, -other.vel.x, -other.acc.x),
            }
        } else {
            let a = Vector2::new(self.a, self.b);
            let pos = other.pos.dot(&a);
            match self.border_type {
                BorderType::Inner => (
                    self.c - ELECTRON_RADIUS - pos,
                    other.vel.dot(&a),
                    other.acc.dot(&a),
                ),
                BorderType::Outer => (
                    pos - ELECTRON_RADIUS - self.c,
                    -other.vel.dot(&a),
                    -other.acc.dot(&a),
                ),
            }
        };
        calc_time_to_border_collision(dist, vel, acc)
    }
}

impl Ord for Border {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.a < other.a {
            std::cmp::Ordering::Less
        } else if self.a > other.a {
            std::cmp::Ordering::Greater
        } else if self.b < other.b {
            std::cmp::Ordering::Less
        } else if self.b > other.b {
            std::cmp::Ordering::Greater
        } else if self.c < other.c {
            std::cmp::Ordering::Less
        } else if self.c > other.c {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for Border {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Border {}

impl PartialEq for Border {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_time_to_collision_left_border() {
        let border = Border::new(1.0, 0.0, 0.0, BorderType::Outer);
        let electron = Electron::new(
            Vector2::new(4.0, 4.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let time = border.calc_time_to_collision(&electron, 800.0);
        assert_eq!(time, 3.5);
    }

    #[test]
    fn calc_time_to_collision_bottom_border() {
        let border = Border::new(0.0, 1.0, 0.0, BorderType::Outer);
        let electron = Electron::new(
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        );
        let time = border.calc_time_to_collision(&electron, 800.0);
        assert_eq!(time, 0.5);
    }

    #[test]
    fn calc_time_to_collision_right_border() {
        let border = Border::new(1.0, 0.0, 800.0, BorderType::Inner);
        let electron = Electron::new(
            Vector2::new(796.0, 596.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let time = border.calc_time_to_collision(&electron, 800.0);
        assert_eq!(time, 3.5);
    }

    #[test]
    fn calc_time_to_collision_top_border() {
        let border = Border::new(0.0, 1.0, 600.0, BorderType::Inner);
        let electron = Electron::new(
            Vector2::new(796.0, 596.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        let time = border.calc_time_to_collision(&electron, 800.0);
        assert_eq!(time, 0.5);
    }
}
