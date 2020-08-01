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

use sfml::graphics::{CircleShape, Color, Font, RectangleShape, RenderTarget,
                     RenderWindow, Shape,Text, Transformable, Vertex, VertexArray, PrimitiveType};

use sfml::system::{Clock, Time, Vector2f, Vector2u, Vector3f};
use sfml::window::{VideoMode, ContextSettings, Event, Key, Style};

const WINDOW_WIDTH:  f32 = 500.0;
const WINDOW_HEIGHT: f32 = 500.0;
const G: f32 = 150.8;

struct Link<'a> {
    length: f32,
    state: [f32; 3],
    extremes_positions: (Vector2f, Vector2f),
    shape: VertexArray,
    color: Color,
    mass: Mass<'a>
}

impl<'a> Link<'a> {
    // create a new Link
    fn new(extremes_positions: (Vector2f, Vector2f), length: f32, color: Color, mass: Mass<'a>) -> Self {
        let mut shape = VertexArray::default();
        shape.set_primitive_type(PrimitiveType::LineStrip);
        shape.append(&Vertex::with_pos_color(extremes_positions.0, color));
        shape.append(&Vertex::with_pos_color(extremes_positions.1, color));
        let theta_0 = 90.0f32.to_radians();
        let state = [theta_0, 0.0, 0.0];
        Self {
            length,
            state,
            extremes_positions,
            shape,
            color,
            mass
        }
    }

    // fn set_position(&mut self, extremes_positions: (Vector2f, Vector2f)) {
    //     self.shape[0].
    // }
}

struct DoublePendulum<'a> {
    link1: Box<Link<'a>>,
    link2: Box<Link<'a>>,
}

impl<'a> DoublePendulum<'a> {
    fn new(link1: Box<Link<'a>>, link2: Box<Link<'a>>) -> Self {
        Self{link1, link2}
    }

    fn update_position(&mut self, dt: f32) {

        let origin_x = WINDOW_WIDTH / 2.0;
        let origin_y = WINDOW_HEIGHT / 3.0;

        let r1 = self.link1.mass.radius;
        let r2 = self.link2.mass.radius;

        let theta1 = self.link1.state[0];
        let theta2 = self.link2.state[0];
        let omega1 = self.link1.state[1];
        let omega2 = self.link2.state[1];
        let m1 = self.link1.mass.mass;
        let m2 = self.link2.mass.mass;
        let thetas_diff = theta1 - theta2;
        let thetas_diff2 = theta1 - 2.0 * theta2;
        let num1 = -G * (2.0 * m1 + m2) * theta1.sin();
        let num2 = -m2 * G * thetas_diff2.sin();
        let num3 = -2.0 * thetas_diff.sin() * m2;
        let num4 = omega2 * omega2 * self.link2.length + omega1 * omega1 * self.link1.length * thetas_diff.cos();
        let den = self.link1.length * (2.0 * m1 + m2 - m2 * (2.0 * (thetas_diff)).cos());
        self.link1.state[2] = (num1 + num2 + num3 * num4) / den;

        let num1 = 2.0 * thetas_diff.sin();
        let num2 = (omega1 * omega1 * self.link1.length * (m1 + m2));
        let num3 = G * (m1 + m2) * theta1.cos();
        let num4 = omega2 * omega2 * self.link2.length * m2 * thetas_diff.cos();
        let den = self.link2.length * (2.0 * m1 + m2 - m2 * (2.0 * (thetas_diff)).cos());
        self.link2.state[2] = (num1 * (num2 + num3 + num4)) / den;

        self.link1.state[1] += self.link1.state[2] * dt;
        self.link2.state[1] += self.link2.state[2] * dt;

        // self.link1.state[1] *= 0.90;
        // self.link2.state[1] *= 0.90;

        self.link1.state[0] += self.link1.state[1] * dt;
        self.link2.state[0] += self.link2.state[1] * dt;

        let x1 = self.link1.length * self.link1.state[0].sin() + origin_x;
        let y1 = self.link1.length * self.link1.state[0].cos() + origin_y;

        let x2 = x1 + self.link2.length * self.link2.state[0].sin();
        let y2 = y1 + self.link2.length * self.link2.state[0].cos();
        let pos1 = Vector2f::new(x1, y1);
        let pos2 = Vector2f::new(x2, y2);
        let origin = Vector2f::new(origin_x, origin_y);

        // self.link1.set_position((origin, pos1));
        // self.link2.set_position((pos1, pos2));
        // println!("pos1: {:?}", pos1);
        self.link1.mass.shape.set_position(pos1 - Vector2f::new(r1, r1));
        self.link2.mass.shape.set_position(pos2 - Vector2f::new(r1, r1));

        self.link1.shape[1].position = pos1;
        self.link2.shape[0].position = pos1;
        self.link2.shape[1].position = pos2;
        // println!("pos2: {:?}", pos2);
    }
}

struct Mass<'a> {
    mass: f32,
    radius: f32,
    position: Vector2f,
    shape: CircleShape<'a>,
    color: Color
}

impl<'a> Mass<'a> {
    fn new(mass: f32, radius: f32, position: Vector2f, color: Color) -> Self {
        let mut shape = CircleShape::new(radius, 100);
        shape.set_position(position);
        shape.set_outline_thickness(3.0);
        shape.set_fill_color(color);
        shape.set_outline_color(Color::BLACK);

        Self{mass, radius, position, shape, color}
    }
}


fn main() {

    let origin_x = WINDOW_WIDTH / 2.0;
    let origin_y = WINDOW_HEIGHT / 3.0;
    // Create the window of the application
    let mut window = RenderWindow::new(
        (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
        "Double pendulum simulation",
        Style::CLOSE,
        &Default::default(),
    );
    //window.set_mouse_cursor_visible(false);
    // window.set_framerate_limit(60);

    window.set_vertical_sync_enabled(true);
    let background_color = Color::rgb(100, 100, 100);

    let origin = Vector2f::new(origin_x, origin_y);
    let pos1   = Vector2f::new(origin_x, origin_y + 100.0);
    let pos2   = Vector2f::new(origin_x, origin_y + 200.0);

    let m1 = 100.0;
    let m2 = 100.0;

    let mass1 = Mass::new(m1, 10.0, pos1 - Vector2f::new(10.0, 10.0), Color::GREEN);
    let link1 = Link::new((origin, pos1), 100.0, Color::RED, mass1);

    let mass2 = Mass::new(m2, 10.0, pos2 - Vector2f::new(10.0, 10.0), Color::WHITE);
    let link2 = Link::new((pos1, pos2), 100.0, Color::BLUE, mass2);

    let mut double_pendulum = DoublePendulum::new(Box::new(link1), Box::new(link2));

    let mut is_running = true;

    let mut clock = Clock::start();

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

        let deltatime = clock.elapsed_time();
        let dt = deltatime.as_seconds();
        clock.restart();
        window.clear(background_color);

        double_pendulum.update_position(dt);
        // println!("theta: {}", double_pendulum.link1.state[0]);
        window.draw(&double_pendulum.link1.shape);
        window.draw(&double_pendulum.link2.shape);
        window.draw(&double_pendulum.link1.mass.shape);
        window.draw(&double_pendulum.link2.mass.shape);
        //
        window.display();
    }
}
