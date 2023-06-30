use std::{cell::RefCell, rc::Weak};

use crate::{
    border::Border, collidable::Collidable, crystal_structure::RcRefCell, electron::Electron,
    ion::Ion,
};
use std::rc::Rc;

#[derive(Clone)]
pub enum Collidables {
    Border(Weak<RefCell<Border>>),
    Ion(Weak<RefCell<Ion>>),
    Electron(Weak<RefCell<Electron>>),
}

impl Collidables {
    pub fn new_b(border: &Rc<RefCell<Border>>) -> Self {
        Collidables::Border(Rc::downgrade(border))
    }

    pub fn new_i(ion: &Rc<RefCell<Ion>>) -> Self {
        Collidables::Ion(Rc::downgrade(ion))
    }

    pub fn new_e(electron: &Rc<RefCell<Electron>>) -> Self {
        Collidables::Electron(Rc::downgrade(electron))
    }

    pub fn empty() -> Self {
        Collidables::Border(Weak::new())
    }

    pub fn calc_time_to_collision(&self, electron: &Electron, width: f64) -> f64 {
        match self {
            Collidables::Border(border) => border
                .upgrade()
                .unwrap()
                .borrow()
                .calc_time_to_collision(electron, width),
            Collidables::Ion(ion) => ion
                .upgrade()
                .unwrap()
                .borrow()
                .calc_time_to_collision(electron),
            Collidables::Electron(electron1) => electron1
                .upgrade()
                .unwrap()
                .borrow()
                .calc_time_to_collision(electron),
        }
    }

    pub fn resolve_collision(&self, electron: &RcRefCell<Electron>, width: f64) {
        electron.borrow_mut().update_stats();
        match self {
            Collidables::Border(border) => border
                .upgrade()
                .unwrap()
                .borrow_mut()
                .bounce(&mut electron.borrow_mut(), width),
            Collidables::Ion(ion) => ion
                .upgrade()
                .unwrap()
                .borrow_mut()
                .bounce(&mut electron.borrow_mut()),
            Collidables::Electron(other) => other
                .upgrade()
                .unwrap()
                .borrow_mut()
                .bounce(&mut electron.borrow_mut()),
        };
    }
}

impl std::cmp::PartialEq for Collidables {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Collidables::Electron(electron1), Collidables::Electron(electron2)) => {
                electron1.upgrade().unwrap().borrow().pos
                    == electron2.upgrade().unwrap().borrow().pos
                    && electron1.upgrade().unwrap().borrow().vel
                        == electron2.upgrade().unwrap().borrow().vel
            }
            (Collidables::Ion(ion1), Collidables::Ion(ion2)) => {
                ion1.upgrade().unwrap().borrow().pos == ion2.upgrade().unwrap().borrow().pos
            }
            (Collidables::Border(border1), Collidables::Border(border2)) => {
                border1.upgrade().unwrap().borrow().a == border2.upgrade().unwrap().borrow().a
                    && border1.upgrade().unwrap().borrow().b
                        == border2.upgrade().unwrap().borrow().b
                    && border1.upgrade().unwrap().borrow().c
                        == border2.upgrade().unwrap().borrow().c
            }
            _ => false,
        }
    }
}

impl std::cmp::Eq for Collidables {}
