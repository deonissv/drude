use std::cell::RefCell;

use std::f64::INFINITY;
use std::rc::{Rc, Weak};

use nalgebra::Vector2;

use crate::border::{Border, BorderType};
use crate::cfg::{ELECTRON_RADIUS, ELEC_ELEC_RADIUS, INIT_ITERATIONS, ION_ELEC_RADIUS, ION_RADIUS};
use crate::collidables::Collidables;
use crate::electron::Electron;
use crate::ion::Ion;

use crate::utils::set_panic_hook;
use js_sys::Math::random;

pub type RcRefCell<T> = Rc<RefCell<T>>;

pub struct CrystalStructure {
    pub x_size: f64,
    pub y_size: f64,
    pub ion_distance: f64,
    pub acc: f64,
    pub borders: Vec<RcRefCell<Border>>,
    pub ions: Vec<RcRefCell<Ion>>,
    pub electrons: Vec<RcRefCell<Electron>>,
    pub next_collision: (Weak<RefCell<Electron>>, Collidables),
    pub time_to_bounce: f64,
    pub elec_left: i32,
    pub elec_right: i32,
}

impl CrystalStructure {
    pub fn new(
        x_size: f64,
        y_size: f64,
        ion_distance: f64,
        init_velocity: f64,
        num_electrons: i32,
    ) -> CrystalStructure {
        let mut crystal_structure = CrystalStructure {
            x_size,
            y_size,
            ion_distance,
            acc: 0.0,
            borders: Vec::new(),
            ions: Vec::new(),
            electrons: Vec::new(),
            next_collision: (Weak::new(), Collidables::empty()),
            time_to_bounce: f64::INFINITY,
            elec_left: 0,
            elec_right: 0,
        };
        set_panic_hook();
        crystal_structure.init_borders();
        crystal_structure.init_ions();
        crystal_structure.init_electrons(init_velocity, num_electrons);

        crystal_structure.update_collidables();
        crystal_structure
    }

    fn init_borders(&mut self) {
        self.borders.push(Rc::new(RefCell::new(Border::new(
            1.0,
            0.0,
            0.0,
            BorderType::Outer,
        ))));
        self.borders.push(Rc::new(RefCell::new(Border::new(
            0.0,
            1.0,
            0.0,
            BorderType::Outer,
        ))));
        self.borders.push(Rc::new(RefCell::new(Border::new(
            1.0,
            0.0,
            self.x_size,
            BorderType::Inner,
        ))));
        self.borders.push(Rc::new(RefCell::new(Border::new(
            0.0,
            1.0,
            self.y_size,
            BorderType::Inner,
        ))));
    }

    fn init_ions(&mut self) {
        let mut x = ((self.x_size - 2.0 * ION_RADIUS) % self.ion_distance) / 2.0 + ION_RADIUS;
        while x + ION_RADIUS <= self.x_size {
            let mut y = ((self.y_size - 2.0 * ION_RADIUS) % self.ion_distance) / 2.0 + ION_RADIUS;
            while y + ION_RADIUS <= self.y_size {
                self.ions
                    .push(Rc::new(RefCell::new(Ion::new(Vector2::new(x, y)))));
                y += self.ion_distance;
            }
            x += self.ion_distance;
        }
    }

    fn init_electrons(&mut self, init_velocity: f64, num_electrons: i32) {
        for _ in 0..num_electrons {
            'a: for _ in 0..INIT_ITERATIONS {
                let x = (random() * (self.x_size - 2.0 * ELECTRON_RADIUS)) + ELECTRON_RADIUS;
                let y = (random() * (self.y_size - 2.0 * ELECTRON_RADIUS)) + ELECTRON_RADIUS;
                let angle = random() * 2.0 * std::f64::consts::PI;
                let vel_x = angle.cos() * init_velocity;
                let vel_y = angle.sin() * init_velocity;
                let electron = Electron::new(
                    Vector2::new(x, y),
                    Vector2::new(vel_x, vel_y),
                    Vector2::new(0.0, 0.0),
                );

                for ion in self.ions.iter() {
                    let dist = (ion.borrow().pos - electron.pos).magnitude();
                    if dist < ELEC_ELEC_RADIUS + ION_RADIUS {
                        continue 'a;
                    };
                }

                for el in self.electrons.iter() {
                    let dist = (el.borrow().pos - electron.pos).magnitude();
                    if dist < 2.0 * ELEC_ELEC_RADIUS {
                        continue 'a;
                    };
                }

                self.electrons.push(Rc::new(RefCell::new(electron)));
                break;
            }
        }
    }

    fn update_collidables(&mut self) {
        let mut min: (Weak<RefCell<Electron>>, Collidables, f64) = (
            Rc::downgrade(&self.electrons[0]),
            Collidables::empty(),
            INFINITY,
        );
        self.electrons.iter().for_each(|electron| {
            let mut collidables: Vec<Collidables> = Vec::new();

            self.borders.iter().for_each(|border| {
                if CrystalStructure::filter_border(&electron.borrow(), &border.borrow()) {
                    collidables.push(Collidables::new_b(border));
                }
            });

            self.ions.iter().for_each(|ion| {
                if CrystalStructure::filter_ion(&electron.borrow(), &ion.borrow()) {
                    collidables.push(Collidables::new_i(ion));
                }
            });
            self.electrons.iter().for_each(|other| {
                if !electron.borrow().eq(&other.borrow()) {
                    collidables.push(Collidables::new_e(other));
                }
            });

            let timed_collidables: Vec<(Collidables, f64)> = collidables
                .iter()
                .map(|c| {
                    (
                        c.clone(),
                        c.calc_time_to_collision(&electron.borrow(), self.x_size),
                    )
                })
                .collect();
            let (collidable, time_to_bounce) = timed_collidables
                .iter()
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();

            electron.borrow_mut().collidable = collidable.clone();
            electron.borrow_mut().time_to_bounce = *time_to_bounce;

            if time_to_bounce < &min.2 {
                min = (Rc::downgrade(electron), collidable.clone(), *time_to_bounce);
            }
        });
        self.next_collision = (min.0, min.1);
        self.time_to_bounce = min.2;
    }

    fn filter_border(electron: &Electron, border: &Border) -> bool {
        match border.border_type {
            BorderType::Inner => {
                electron.vel.x > 0.0 && border.a == 1.0 && border.c > 0.0
                    || electron.vel.y > 0.0 && border.b == 1.0 && border.c > 0.0
            }
            BorderType::Outer => {
                electron.vel.x < 0.0 && border.a == 1.0 && border.c == 0.0
                    || electron.vel.y < 0.0 && border.b == 1.0 && border.c == 0.0
            }
        }
    }

    fn filter_ion(electron: &Electron, ion: &Ion) -> bool {
        if electron.vel.x > 0.0 && electron.vel.y > 0.0 {
            (electron.pos.x - ION_ELEC_RADIUS <= ion.pos.x)
                && (electron.pos.y - ION_ELEC_RADIUS <= ion.pos.y)
        } else if electron.vel.x > 0.0 && electron.vel.y < 0.0 {
            (electron.pos.x - ION_ELEC_RADIUS <= ion.pos.x)
                && (electron.pos.y + ION_ELEC_RADIUS >= ion.pos.y)
        } else if electron.vel.x < 0.0 && electron.vel.y > 0.0 {
            (electron.pos.x + ION_ELEC_RADIUS >= ion.pos.x)
                && (electron.pos.y - ION_ELEC_RADIUS <= ion.pos.y)
        } else if electron.vel.x < 0.0 && electron.vel.y < 0.0 {
            (electron.pos.x + ION_ELEC_RADIUS >= ion.pos.x)
                && (electron.pos.y + ION_ELEC_RADIUS >= ion.pos.y)
        } else if electron.vel.x == 0.0 && electron.vel.y > 0.0 {
            electron.pos.y < ion.pos.y
                && electron.pos.x - ION_ELEC_RADIUS - 1.0 <= ion.pos.x
                && ion.pos.x <= electron.pos.x + ION_ELEC_RADIUS + 1.0
        } else if electron.vel.x == 0.0 && electron.vel.y < 0.0 {
            electron.pos.y > ion.pos.y
                && electron.pos.x - ION_ELEC_RADIUS - 1.0 <= ion.pos.x
                && ion.pos.x <= electron.pos.x + ION_ELEC_RADIUS + 1.0
        } else if electron.vel.y == 0.0 && electron.vel.x > 0.0 {
            electron.pos.x < ion.pos.x
                && electron.pos.y - ION_ELEC_RADIUS - 1.0 <= ion.pos.y
                && ion.pos.y <= electron.pos.x + ION_ELEC_RADIUS + 1.0
        } else if electron.vel.y == 0.0 && electron.vel.x < 0.0 {
            electron.pos.x > ion.pos.x
                && electron.pos.y - ION_ELEC_RADIUS - 1.0 <= ion.pos.y
                && ion.pos.y <= electron.pos.x + ION_ELEC_RADIUS + 1.0
        } else {
            false
        }
    }

    pub fn update(&mut self, acc: f64, supp: f64) {
        let mut time: f64 = 1.0;

        if self.acc != acc {
            self.acc = acc;
            let acc_vec = Vector2::new(acc, 0.0);
            self.electrons.iter().for_each(|electron| {
                electron.borrow_mut().acc = acc_vec;
            });
            self.update_collidables();
        }

        while time > 0.0 {
            if self.time_to_bounce > time {
                self.electrons
                    .iter()
                    .for_each(|electron| electron.borrow_mut().update(time, supp));
                self.time_to_bounce -= time;
                return;
            }

            self.electrons
                .iter()
                .for_each(|electron| electron.borrow_mut().update(self.time_to_bounce, supp));

            self.update_elec_stats(&self.next_collision.0.upgrade().unwrap().borrow());
            self.next_collision
                .1
                .resolve_collision(&self.next_collision.0.upgrade().unwrap(), self.x_size);
            time -= self.time_to_bounce;
            self.update_collidables();
        }
    }

    fn update_elec_stats(&mut self, electron: &Electron) {
        match self.next_collision.1 {
            Collidables::Border(_) if electron.vel.x > 0.0 => {
                self.elec_left += 1;
            }
            Collidables::Border(_) if electron.vel.x < 0.0 => {
                self.elec_right += 1;
            }
            _ => {}
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_cs() -> CrystalStructure {
        CrystalStructure {
            x_size: 800.0,
            y_size: 600.0,
            ion_distance: 100.0,
            acc: 0.0,
            borders: Vec::new(),
            ions: Vec::new(),
            electrons: Vec::new(),
            next_collision: (Weak::new(), Collidables::empty()),
            time_to_bounce: INFINITY,
            elec_left: 0,
            elec_right: 0,
        }
    }

    fn borders() -> Vec<Border> {
        vec![
            Border::new(1.0, 0.0, 0.0, BorderType::Outer),
            Border::new(0.0, 1.0, 0.0, BorderType::Outer),
            Border::new(1.0, 0.0, 800.0, BorderType::Inner),
            Border::new(0.0, 1.0, 600.0, BorderType::Inner),
        ]
    }

    #[test]
    fn filter_border_top() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[3]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[1]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[2]));
    }

    #[test]
    fn filter_border_left() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[2]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[1]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[3]));
    }

    #[test]
    fn filter_border_bottom() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[1]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[2]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[3]));
    }

    #[test]
    fn filter_border_right() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[0]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[1]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[2]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[3]));
    }

    #[test]
    fn filter_border_top_left() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(2.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[2]));
        assert!(CrystalStructure::filter_border(&electron, &borders[3]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[1]));
    }

    #[test]
    fn filter_border_top_right() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(-2.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(CrystalStructure::filter_border(&electron, &borders[3]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[1]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[2]));
    }

    #[test]
    fn filter_border_bottom_left() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(-2.0, -2.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(CrystalStructure::filter_border(&electron, &borders[1]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[2]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[3]));
    }

    #[test]
    fn filter_border_bottom_right() {
        let borders = borders();
        let electron = Electron::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(2.0, -2.0),
            Vector2::new(0.0, 0.0),
        );
        assert!(CrystalStructure::filter_border(&electron, &borders[1]));
        assert!(CrystalStructure::filter_border(&electron, &borders[2]));

        assert!(!CrystalStructure::filter_border(&electron, &borders[0]));
        assert!(!CrystalStructure::filter_border(&electron, &borders[3]));
    }

    #[test]
    fn filter_ion_top() {
        let electron = Electron::new(
            Vector2::new(50.0, 50.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        let ion_top = Ion::new(Vector2::new(50.0, 100.0));
        let ion_left = Ion::new(Vector2::new(100.0, 50.0));
        let ion_bottom = Ion::new(Vector2::new(50.0, 0.0));
        let ion_right = Ion::new(Vector2::new(0.0, 50.0));

        assert!(CrystalStructure::filter_ion(&electron, &ion_top));

        assert!(!CrystalStructure::filter_ion(&electron, &ion_left));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_right));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_bottom));
    }

    #[test]
    fn filter_ion_left() {
        let electron = Electron::new(
            Vector2::new(50.0, 50.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        );
        let ion_top = Ion::new(Vector2::new(50.0, 100.0));
        let ion_left = Ion::new(Vector2::new(0.0, 50.0));
        let ion_bottom = Ion::new(Vector2::new(50.0, 0.0));
        let ion_right = Ion::new(Vector2::new(100.0, 50.0));

        assert!(CrystalStructure::filter_ion(&electron, &ion_left));

        assert!(!CrystalStructure::filter_ion(&electron, &ion_top));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_right));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_bottom));
    }

    #[test]
    fn filter_ion_bottom() {
        let electron = Electron::new(
            Vector2::new(50.0, 50.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        );
        let ion_top = Ion::new(Vector2::new(50.0, 100.0));
        let ion_left = Ion::new(Vector2::new(0.0, 50.0));
        let ion_bottom = Ion::new(Vector2::new(50.0, 0.0));
        let ion_right = Ion::new(Vector2::new(100.0, 50.0));

        assert!(CrystalStructure::filter_ion(&electron, &ion_bottom));

        assert!(!CrystalStructure::filter_ion(&electron, &ion_top));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_right));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_left));
    }

    #[test]
    fn filter_ion_top_left() {
        let electron = Electron::new(
            Vector2::new(50.0, 50.0),
            Vector2::new(-2.0, 2.0),
            Vector2::new(0.0, 0.0),
        );
        let ion_top_left = Ion::new(Vector2::new(0.0, 100.0));
        let ion_top_right = Ion::new(Vector2::new(100.0, 100.0));
        let ion_bottom_left = Ion::new(Vector2::new(0.0, 0.0));
        let ion_bottom_right = Ion::new(Vector2::new(100.0, 0.0));

        assert!(CrystalStructure::filter_ion(&electron, &ion_top_left));

        assert!(!CrystalStructure::filter_ion(&electron, &ion_top_right));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_bottom_left));
        assert!(!CrystalStructure::filter_ion(&electron, &ion_bottom_right));
    }

    #[test]
    fn border_direct_bounce_left_border() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(4.0, 4.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);
        cs.update(0.0, 0.0);
        cs.update(0.0, 0.0);
        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 796.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 4.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, -2.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 0.0);
    }

    #[test]
    fn border_direct_bounce_bottom_border() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 4.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 4.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 2.0);
    }

    #[test]
    fn border_direct_bounce_right_border() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(796.0, 596.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);
        cs.update(0.0, 0.0);
        cs.update(0.0, 0.0);
        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 4.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 596.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 2.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 0.0);
    }

    #[test]
    fn border_direct_bounce_top_border() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(796.0, 596.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 796.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 596.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, -2.0);
    }

    #[test]
    fn ion_direct_bounce_left() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.ions
            .push(Rc::new(RefCell::new(Ion::new(Vector2::new(100.0, 100.0)))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(86.0, 100.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 86.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 100.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, -2.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 0.0);
    }

    #[test]
    fn ion_direct_bounce_right() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.ions
            .push(Rc::new(RefCell::new(Ion::new(Vector2::new(100.0, 100.0)))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(114.0, 100.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 114.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 100.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 2.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 0.0);
    }

    #[test]
    fn ion_direct_bounce_top() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.ions
            .push(Rc::new(RefCell::new(Ion::new(Vector2::new(100.0, 100.0)))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 114.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 114.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 2.0);
    }

    #[test]
    fn ion_direct_bounce_bottom() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.ions
            .push(Rc::new(RefCell::new(Ion::new(Vector2::new(100.0, 100.0)))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 86.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 86.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, -2.0);
    }

    #[test]
    fn electron_direct_bounce_left() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 100.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(93.0, 100.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 101.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 100.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 2.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 0.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 94.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 100.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, 0.0);
    }

    #[test]
    fn electron_direct_bounce_right() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 100.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(107.0, 100.0),
            Vector2::new(-2.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 99.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 100.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, -2.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 0.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 106.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 100.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, 0.0);
    }

    #[test]
    fn electron_direct_bounce_top() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 100.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 107.0),
            Vector2::new(0.0, -2.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 99.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, -2.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 106.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, 0.0);

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 97.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, -2.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 106.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, 0.0);
    }

    #[test]
    fn electron_direct_bounce_bottom() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 100.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 93.0),
            Vector2::new(0.0, 2.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 101.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, 2.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 94.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, 0.0);
    }

    #[test]
    fn electron_direct_bounce_top_chase() {
        let mut cs = get_cs();
        cs.init_borders();
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 100.0),
            Vector2::new(0.0, -1.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.electrons.push(Rc::new(RefCell::new(Electron::new(
            Vector2::new(100.0, 107.0),
            Vector2::new(0.0, -3.0),
            Vector2::new(0.0, 0.0),
        ))));
        cs.update_collidables();

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 98.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, -3.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 105.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, -1.0);

        cs.update(0.0, 0.0);

        assert_eq!(cs.electrons[0].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[0].borrow().pos.y, 95.0);
        assert_eq!(cs.electrons[0].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[0].borrow().vel.y, -3.0);

        assert_eq!(cs.electrons[1].borrow().pos.x, 100.0);
        assert_eq!(cs.electrons[1].borrow().pos.y, 104.0);
        assert_eq!(cs.electrons[1].borrow().vel.x, 0.0);
        assert_eq!(cs.electrons[1].borrow().vel.y, -1.0);
    }
}
