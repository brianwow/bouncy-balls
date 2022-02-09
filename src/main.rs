#![allow(unused)]

use cairo::ffi::{cairo_create, cairo_surface_t, cairo_t};
use cairo_sys::{cairo_arc, cairo_fill, cairo_pop_group_to_source, cairo_surface_flush};
use cairo_sys::{
    cairo_destroy, cairo_paint, cairo_push_group, cairo_set_source_rgb, cairo_surface_destroy,
    cairo_xlib_surface_create,
};
use rand::Rng;
use std::ops::IndexMut;
use std::{f64::consts::PI, thread, time::Duration};
use x11::xlib::{
    XCloseDisplay, XCreateSimpleWindow, XDefaultRootWindow, XDefaultScreen, XDefaultVisual,
    XMapWindow, XOpenDisplay,
};

const HEIGHT: u32 = 800;
const WIDTH: u32 = 1024;
const WAIT: u64 = 5000;
const NUM_BALLS: usize = 8;

#[derive(Debug)]
struct Ball {
    vx: f64,
    vy: f64,
    m: f64,
    r: f64,
    x: f64,
    y: f64,
}

fn gen_rand(range: std::ops::Range<u32>) -> f64 {
    rand::thread_rng().gen_range::<u32, _>(range) as f64
}

fn init_ball() -> Vec<Ball> {
    (0..NUM_BALLS)
        .map(|_| Ball {
            vx: gen_rand(0..400) - 200.0,
            vy: gen_rand(0..400) - 200.0,
            m: gen_rand(0..80) + 25.0,
            r: gen_rand(0..80) + 25.0,
            x: gen_rand(0..WIDTH),
            y: gen_rand(0..HEIGHT),
        })
        .collect::<Vec<Ball>>()
}

fn update(dt: f64, balls: &mut Vec<Ball>) -> bool {
    let mut min = 0f64;
    for a in 0..NUM_BALLS {
        let mut ball_iter = balls.iter_mut();
        let mut b = ball_iter.nth(a).unwrap();
        for c in 0..NUM_BALLS {
            if let Some(b1) = ball_iter.nth(c) {
                if (b.x - b1.x).abs() < b.r + b1.r && (b.y - b1.y).abs() < b.r + b1.r {
                    let dist = ((b.x - b1.x).powf(2.0) + (b.y - b1.y).powf(2.0)).sqrt();
                    if dist < b.r + b1.r {
                        let theta = (b.y - b1.y).atan2(b.x - b1.x);
                        let mag = 1000.0 * b.m * b1.m / dist;
                        let diff = (b.r + b1.r - dist) / 2.0;
                        b1.vx -= mag * theta.cos() / b1.m;
                        b1.vy -= mag * theta.sin() / b1.m;
                        b.vx += mag * theta.cos() / b.m;
                        b.vy += mag * theta.sin() / b.m;
                        b1.x -= diff * theta.cos();
                        b1.y -= diff * theta.sin();
                        b.x += diff * theta.cos();
                        b.y += diff * theta.sin();
                    }
                }
            }
        }
        let fric = 25.0 * dt;
        if b.vx > 0.0 {
            b.vx -= if b.vx > fric { fric } else { b.vx };
        }
        if b.vx < 0.0 {
            b.vx -= if b.vx < -fric { -fric } else { b.vy };
        }
        if b.vy > 0.0 {
            b.vy -= if b.vy > fric { fric } else { b.vy };
        }
        if b.vy < 0.0 {
            b.vy -= if b.vy < -fric { -fric } else { b.vy };
        }
        if b.x + b.r >= WIDTH as f64 {
            b.vx = -(b.vx.abs());
        }
        if b.y + b.r >= HEIGHT as f64 {
            b.vy = -(b.vy.abs());
        }
        if b.x - b.r <= 0.0 {
            b.vx = b.vx.abs();
        }
        if b.y - b.r <= 0.0 {
            b.vy = b.vy.abs();
        }
        if b.vx > min {
            min = b.vx;
        }
        if b.vy > min {
            min = b.vy;
        }
        b.x += dt * b.vx;
        b.y += dt * b.vy;
    }
    min > 0.0
}

fn r#loop(cr: *mut cairo_t, surface: *mut cairo_surface_t, balls: &mut Vec<Ball>) -> bool {
    let repeat: bool = update(WAIT as f64 / 1000000.0, balls);
    unsafe {
        cairo_push_group(cr);
        cairo_set_source_rgb(cr, 1.0, 1.0, 1.0);
        cairo_paint(cr);
        cairo_set_source_rgb(cr, 0.0, 0.0, 0.0);
        (0..NUM_BALLS).for_each(|a| {
            cairo_arc(cr, balls[a].x, balls[a].y, balls[a].r, 0.0, PI * 2.0);
            cairo_fill(cr);
        });
        cairo_pop_group_to_source(cr);
        cairo_paint(cr);
        cairo_surface_flush(surface);
    }
    repeat
}

fn main() {
    unsafe {
        let d = XOpenDisplay(std::ptr::null::<i8>());
        let screen = XDefaultScreen(d);
        let visual = XDefaultVisual(d, screen);
        let draw = XCreateSimpleWindow(d, XDefaultRootWindow(d), 0, 0, WIDTH, HEIGHT, 0, 0, 0);
        XMapWindow(d, draw);
        let surface = cairo_xlib_surface_create(d, draw, visual, WIDTH as i32, HEIGHT as i32);
        let cr = cairo_create(surface);
        let mut balls = init_ball();
        while r#loop(cr, surface, &mut balls) {
            thread::sleep(Duration::from_micros(WAIT));
        }
        thread::sleep(Duration::from_secs(2));
        cairo_surface_destroy(surface);
        cairo_destroy(cr);
        XCloseDisplay(d);
    }
}
