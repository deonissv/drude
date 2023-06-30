extern crate nalgebra as na;
use std::f64::INFINITY;

use na::Vector2;
use roots::{find_roots_quadratic, find_roots_quartic};

use crate::cfg::EPSILON;

// Calculates the time till bounce of two objects.
// @param pos1 The initial position of the first object.
// @param vel1 The velocity of the first object.
// @param pos2 The initial position of the second object.
// @param vel2 The velocity of the second object.
// @param r_sum The sum of the radiuses of the two objects.
// @returns The time till bounce.
pub fn calc_time_to_collision(
    pos1: Vector2<f64>,
    vel1: Vector2<f64>,
    acc1: Vector2<f64>,
    pos2: Vector2<f64>,
    vel2: Vector2<f64>,
    acc2: Vector2<f64>,
    r_sum: f64,
) -> f64 {
    let d_pos = pos2 - pos1;
    let d_vel = vel2 - vel1;
    let d_acc = acc2 - acc1;

    if d_pos.angle(&d_vel) < 0.0 {
        return INFINITY;
    }

    let acc_dot = d_acc.dot(&d_acc);
    let mut roots = if acc_dot == 0.0 {
        let a = d_vel.dot(&d_vel);
        let b = 2.0 * d_vel.dot(&d_pos);
        let c = d_pos.dot(&d_pos) - r_sum.powi(2);
        find_roots_quadratic(a, b, c).as_ref().to_vec()
    } else {
        let a = acc_dot / 4.0;
        let b = d_vel.dot(&d_acc);
        let c = d_vel.dot(&d_vel) + d_pos.dot(&d_acc);
        let d = 2.0 * d_pos.dot(&d_vel);
        let e = d_pos.dot(&d_pos) - r_sum.powi(2);
        find_roots_quartic(a, b, c, d, e).as_ref().to_vec()
    };
    roots.retain(|&x| x > EPSILON);
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if roots.is_empty() {
        return INFINITY;
    }
    roots[0]
}

pub fn calc_time_to_border_collision(dist: f64, vel: f64, acc: f64) -> f64 {
    let mut roots = find_roots_quadratic(acc / 2.0, vel, -dist)
        .as_ref()
        .to_vec();
    roots.retain(|&x| x > EPSILON);
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if roots.is_empty() {
        return INFINITY;
    }
    roots[0]
}

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
