//-------------------------------------------------------------------------
// @file main.rs
//
// @date 07/12/20 15:44:36
// @author Martin Noblia
// @email mnoblia@disroot.org
//
// @brief
//
// @detail
//
//  Licence:
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
//--------------------------------------------------------------------------
/// Double pendulum animation
// TODO(elsuizo:2020-07-14): list of:
// - [  ] implementar RK(Runge-kutta)
// - [  ] implementar el game-loop para que podamos pausar o cambiar algo
// - [  ] hacer que participe el mouse (puede ser moviendo la posicion inicial)
//-------------------------------------------------------------------------
//                        crates imports
//-------------------------------------------------------------------------
extern crate sfml;
//-------------------------------------------------------------------------
//                        includes
//-------------------------------------------------------------------------
use sfml::graphics::{CircleShape, Color, Font, RectangleShape, RenderTarget,
                     RenderWindow, Shape,Text, Transformable};
use sfml::system::{Clock, Time, Vector2f, Vector2u};
use sfml::window::{VideoMode, ContextSettings, Event, Key, Style};

const WINDOW_WIDTH:  f32 = 500.0;
const WINDOW_HEIGHT: f32 = 500.0;

struct Link<'a> {
    length: f32,
    theta: f32,
    position: Vector2f,
    shape: RectangleShape<'a>,
    color: Color,
    mass: f32,
    joint: Joint<'a>
}

struct Joint<'a> {
    radius: f32,
    position: Vector2f,
    shape: CircleShape<'a>,
    color: Color
}

struct DoublePendulum<'a> {
    link1: Box<Link<'a>>,
    link2: Box<Link<'a>>,
}

impl<'a> Joint<'a> {
    fn new(radius: f32, x: f32, y: f32, color: Color) -> Self {
        let position = Vector2f::new(x, y);
        let mut shape = CircleShape::new(radius, 100);
        let origin   = Vector2f::new(5.0, 0.0);
        shape.set_origin(origin);
        shape.set_position(position);
        shape.set_outline_thickness(3.0);
        shape.set_fill_color(color);
        shape.set_outline_color(Color::BLACK);

        Self{radius, position, shape, color}
    }
}

impl<'a> DoublePendulum<'a> {
    fn new(link1: Box<Link<'a>>, link2: Box<Link<'a>>) -> Self {
        Self{link1, link2}
    }

    fn move_links(&mut self, theta1: f32, theta2: f32) {
        // TODO(elsuizo:2020-06-12): no se para que quiero guardar las thetas :)
        self.link1.theta = theta1;
        self.link2.theta = theta2;
        let x = 250.0 - self.link1.length * theta1.to_radians().sin();
        let y = 250.0 + self.link1.length * theta1.to_radians().cos();
        let new_position = Vector2f::new(x, y);
        // NOTE(elsuizo:2020-07-14): el joint tiene un pequenio offset para que se vea bien
        let joint_new_position = Vector2f::new(x, y - 5.0);
        self.link1.shape.set_rotation(theta1);
        self.link2.shape.set_position(new_position);
        // rotamos a el link2
        self.link2.shape.set_rotation(theta2);
        self.link2.joint.shape.set_position(joint_new_position);
    }

}

impl<'a> Link<'a> {
    // create a new Link
    fn new(x: f32, y: f32, length: f32, color: Color, mass: f32, theta: f32, joint: Joint<'a>) -> Self {
        let mut shape = RectangleShape::new();
        let position = Vector2f::new(x, y);
        let size     = Vector2f::new(10.0, length);
        let origin   = Vector2f::new(5.0, 0.0);
        // link shape and geometric initialization
        shape.set_size(size);
        shape.set_origin(origin);
        shape.set_position(position);
        shape.set_outline_thickness(3.0);
        shape.set_fill_color(color);
        shape.set_outline_color(Color::BLACK);

        Self {
            length,
            theta,
            position,
            shape,
            color,
            mass,
            joint
        }
    }
}

fn show_shapes(d: &DoublePendulum, w: &mut RenderWindow, c: Color) {
    w.clear(c);
    w.draw(&d.link1.shape);
    w.draw(&d.link2.shape);
    w.draw(&d.link1.joint.shape);
    w.draw(&d.link2.joint.shape);

    w.display();
}

//-------------------------------------------------------------------------
//                        main
//-------------------------------------------------------------------------
fn main() {

    let origin_x = WINDOW_WIDTH / 2.0;
    let origin_y = WINDOW_HEIGHT / 2.0;
    let mut theta1: f32 = 90.0;
    let mut theta2: f32 = 0.0;
    let mut omega1: f32 = 0.0250;
    let mut omega2: f32 = 0.0250;
    let mut omega1_dot: f32 = 0.01;
    let mut omega2_dot: f32 = 0.01;
    // Create the window of the application
    let mut window = RenderWindow::new(
        (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
        "Double pendulum animation",
        Style::CLOSE,
        &Default::default(),
    );
    //window.set_mouse_cursor_visible(false);
    window.set_framerate_limit(60);

    window.set_vertical_sync_enabled(true);
    let background_color = Color::rgb(100, 100, 100);

    let joint1 = Joint::new(5.0, origin_x, origin_y -5.0, Color::GREEN);
    let link1 = Link::new(origin_x, origin_y, 100.0, Color::RED, 1.0, 0.0, joint1);

    let joint2 = Joint::new(5.0, origin_x + link1.length, origin_y -5.0, Color::WHITE);
    let link2 = Link::new(origin_x, origin_y + link1.length, 100.0, Color::BLUE, 1.0, 0.0, joint2);

    let mut double_pendulum = DoublePendulum::new(Box::new(link1), Box::new(link2));
    let mut clock = Clock::start();
    let mut is_running = true;
    //-------------------------------------------------------------------------
    //                        loop
    //-------------------------------------------------------------------------
    let mut i = 0;
    let g = 100.81;

    let mut clock = Clock::start();
    let ai_time = Time::seconds(0.1);

    loop {

        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed | Event::KeyPressed {code: Key::Escape, ..} => return,
                Event::KeyPressed {code: Key::Space, ..} if !is_running => {
                    is_running = true;
                    //clock.restart();
                    println!("space!!!");
                }
                _ => {}
            }
        }
        // NOTE(elsuizo:2020-06-12): me parece que es mejor hacerlo con RK
        omega1 += omega1_dot;
        omega2 += omega2_dot;
        theta1 += omega1;
        theta2 += omega2;

        let m1 = 200.0;
        let m2 = 200.0;
        let l1 = 100.0;
        let l2 = 100.0;

        let num1 = -g * (2.0 * m1 + m2) * theta1.sin() - m2 * g * (theta1 - 2.0 * theta2).sin() - 2.0 * (theta1 - theta2).sin() * m2 * (omega2 * omega2 * l2 + omega1 * omega1 * l1 * (theta1 - theta2).cos());
        let den1 = l1 * (2.0 * m1 + m2 - m2 * (2.0 * theta1 - 2.0 * theta2).cos());
        let omega1_dot = num1 / den1;

        let num2 = 2.0 * (theta1 - theta2).sin() * (omega1 * omega1 * l1 * (m1 + m2) + g * (m1 + m2) * theta1.cos() + omega2 * omega2 * l2 * m2 * (theta1 - theta2).cos());
        let den2 = l2 * (2.0 * m1 + m2 - m2 * (2.0 * theta1 - 2.0 * theta2).cos());
        let omega2_dot = num2 / den2;
        // let mouse_position = window.mouse_position();

        // let delta_time = clock.restart().as_seconds();
        // println!("(theta1, theta2): ({}, {})", theta1, theta2);

        double_pendulum.move_links(theta1, theta2);


        // window.clear(background_color);
        show_shapes(&double_pendulum, &mut window, background_color);
        // window.draw(&double_pendulum.link1.shape);
        // window.draw(&double_pendulum.link2.shape);
        // window.draw(&double_pendulum.link1.joint.shape);
        // window.draw(&double_pendulum.link2.joint.shape);
        //
        // window.display();
    }
}
