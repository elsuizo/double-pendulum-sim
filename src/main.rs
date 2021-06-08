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
// - [X] implement Runge-kutta
// - [ ] implement game loop to pause and change parameters
//      - [X] pause the simulation
//      - [ ] when pause pos2 should follow the mouse pointer
// - [X] plot the pos2 trayectory
// - [ ] plot the acceleration and velocities
// - [ ] live change parameters when the simulation pause
//      - [ ] the radius of the masses should change proportionally
// - [ ] text rendering like energy value and velocities values
// - [ ] save the configuration in a file to later use like a .json, yaml or csv or toml or ...
//-------------------------------------------------------------------------
//                        crates imports
//-------------------------------------------------------------------------
use sfml::graphics::{CircleShape, Color, RenderTarget,
                     RenderWindow, Shape, Transformable, Vertex, VertexArray, PrimitiveType, Font, Text};

use sfml::system::{Clock, Vector2f};
use sfml::window::{Event, Key, Style};

use static_math::{M22, m22_new, V4, V2};
use static_math::traits::LinearAlgebra;

const WINDOW_WIDTH:  f32 = 500.0;
const WINDOW_HEIGHT: f32 = 500.0;
const ORIGIN_X: f32 = WINDOW_WIDTH / 2.0;
const ORIGIN_Y: f32 = WINDOW_HEIGHT / 3.0;
const G: f32 = 9.81;

struct Link<'a> {
    length: f32,
    states: [f32; 2],
    shape: VertexArray,
    color: Color,
    mass: Mass<'a>
}

impl<'a> Link<'a> {
    /// Create a new link
    fn new(length: f32, color: Color, mass: Mass<'a>, [theta_0, omega_0]: [f32; 2]) -> Self {
        // this create the lines
        let mut shape = VertexArray::default();
        shape.set_primitive_type(PrimitiveType::LineStrip);
        shape.append(&Vertex::with_pos_color(Vector2f::new(ORIGIN_X, ORIGIN_Y), color));
        shape.append(&Vertex::with_pos_color(Vector2f::new(0.0, 0.0), color));
        Self {
            length,
            states: [theta_0, omega_0],
            shape,
            color,
            mass
        }
    }
}

struct DoublePendulum<'a> {
    link1: Box<Link<'a>>,
    link2: Box<Link<'a>>,
    path: Vec<CircleShape<'a>>,
    states: V4<f32>
}

impl<'a> DoublePendulum<'a> {

    fn new(link1: Box<Link<'a>>, link2: Box<Link<'a>>) -> Self {
        let path: Vec<CircleShape> = Vec::new();
        let theta1_0 = link1.states[0];
        let theta2_0 = link2.states[0];
        let omega1_0 = link1.states[1];
        let omega2_0 = link2.states[1];
        // initial conditions to the system
        let states = V4::new_from(omega1_0, omega2_0, theta1_0, theta2_0);
        Self{link1, link2, path, states}
    }

    // NOTE(elsuizo:2021-05-25): wide code is better code :)
    fn system(&self, states: V4<f32>) -> V4<f32> {
        let m1 = self.link1.mass.mass;
        let m2 = self.link2.mass.mass;
        let l1 = self.link1.length;
        let l2 = self.link2.length;
        let (omega1, omega2, theta1, theta2) = (states[0], states[1], states[2], states[3]);

        let m = m22_new!(         (m1 + m2) * l1     , m2 * l2 * (theta1 - theta2).cos();
                         l1 * (theta1 - theta2).cos(),          l2                      );

        let f = V2::new_from(-m2 * l2 * omega2 * omega2 * (theta1 - theta2).sin() - (m1 + m2) * G * theta1.sin(),
                             l1 * omega1 * omega1 * (theta1 - theta2).sin() - G * theta2.sin());

        let acceleration = m.inverse().expect("no inverse!!!") * f;
        V4::new_from(acceleration[0], acceleration[1], omega1, omega2)
    }

    fn runge_kutta(&self, states: V4<f32>, dt: f32) -> V4<f32> {
        let k1 = self.system(states);
        let k2 = self.system(states + 0.5 * k1 * dt);
        let k3 = self.system(states + 0.5 * k2 * dt);
        let k4 = self.system(states + k3 * dt);

        // return dt * G(y)
        dt * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0
    }

    fn get_position(&self) -> (Vector2f, Vector2f) {
        let scale = 50.0;
        let l1 = self.link1.length;
        let l2 = self.link2.length;
        let theta1 = self.states[2];
        let theta2 = self.states[3];
        let x1 = l1 * scale * theta1.sin() + ORIGIN_X;
        let y1 = l1 * scale * theta1.cos() + ORIGIN_Y;
        let x2 = x1 + l2 * scale * theta2.sin();
        let y2 = y1 + l2 * scale * theta2.cos();

        (Vector2f::new(x1, y1), Vector2f::new(x2, y2))
    }

    fn update_position(&mut self, dt: f32) {

        let r1 = self.link1.mass.radius;
        // integration
        self.states += self.runge_kutta(self.states, dt);
        let (pos1, pos2) = self.get_position();

        self.link1.mass.shape.set_position(pos1 - Vector2f::new(r1, r1));
        self.link2.mass.shape.set_position(pos2 - Vector2f::new(r1, r1));
        self.link1.shape[1].position = pos1;
        self.link2.shape[0].position = pos1;
        self.link2.shape[1].position = pos2;

        // path drawing
        let mut circle = CircleShape::new(1.5, 40);
        circle.set_position(pos2);
        if self.path.len() > 200 {
            self.path.clear();
        }
        self.path.push(circle);
    }
}

struct Mass<'a> {
    mass: f32,
    radius: f32,
    shape: CircleShape<'a>,
    color: Color
}

impl<'a> Mass<'a> {
    fn new(mass: f32, radius: f32, color: Color) -> Self {
        let mut shape = CircleShape::new(radius, 100);
        shape.set_outline_thickness(3.0);
        shape.set_fill_color(color);
        shape.set_outline_color(Color::BLACK);

        Self{mass, radius, shape, color}
    }
}

fn main() {

    // Create the window of the application
    let mut window = RenderWindow::new(
        (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
        "Double pendulum simulation",
        Style::CLOSE,
        &Default::default(),
    );

    let font = Font::from_file("resources/sansation.ttf").unwrap();
    let mut simulation_running_text = Text::new("simulation running", &font, 16);
    let mut simulation_paussed_text = Text::new("simulation paused", &font, 16);

    window.set_vertical_sync_enabled(true);
    let background_color = Color::BLACK;

    let m1 = 1.0;
    let l1 = 2.0;

    let link1_states_0 = [90f32.to_radians(), 2.0];
    let mass1 = Mass::new(m1, 10.0, Color::GREEN);
    let link1 = Link::new(l1, Color::RED, mass1, link1_states_0);

    let m2 = 3.0;
    let l2 = 2.0;

    let link2_states_0 = [130f32.to_radians(), 0.0];
    let mass2 = Mass::new(m2, 10.0, Color::RED);
    let link2 = Link::new(l2, Color::BLUE, mass2, link2_states_0);

    let mut double_pendulum = DoublePendulum::new(Box::new(link1), Box::new(link2));

    let mut is_running = true;

    let mut clock = Clock::start();

    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed | Event::KeyPressed {code: Key::Escape, ..} => return,
                Event::KeyPressed {code: Key::Space, ..} => {
                    if is_running {
                        clock.restart();
                        is_running = false;
                    } else {
                        clock.restart();
                        is_running = true;
                    }
                }
                _ => {}
            }
        }

        if is_running {
            window.draw(&simulation_running_text);
            let deltatime = clock.elapsed_time();
            let dt = deltatime.as_seconds();
            clock.restart();
            window.clear(background_color);

            double_pendulum.update_position(dt);

            window.draw(&double_pendulum.link1.shape);
            window.draw(&double_pendulum.link2.shape);
            window.draw(&double_pendulum.link1.mass.shape);
            window.draw(&double_pendulum.link2.mass.shape);
            for c in &mut double_pendulum.path {
                let mut color = c.fill_color();
                if color.a > 4 {
                    color.a -= 4;
                }
                color.r = 100;
                color.g = 10;
                color.b = 100;
                c.set_fill_color(color);
                window.draw(c);
            }
        } else {
            window.draw(&simulation_paussed_text);
        }
        window.display();
    }
}
