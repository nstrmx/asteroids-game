use rand::*;
use raylib::prelude::*;

use crate::{check_collision_circle_triangle, Circle, Event, Textures, Triangle, SCREEN_HEIGHT, SCREEN_WIDTH};


pub type EntityId = usize;

#[derive(Debug)]
pub enum Entity {
    Player(Box<Player>),
    Enemy(Box<Asteroid>),
    Projectile(Box<Lazer>),
    Star(Box<Star>),
    Indestructible,
}

impl Entity {
    pub fn check_collision(&self, other: &Entity) -> bool {
        match (self, other) {
            (Self::Player(e1), Self::Enemy(e2)) | (Self::Enemy(e2), Self::Player(e1)) => {
                check_collision_circle_triangle(
                    &Circle::from_rect(&e2.rect),
                    &Triangle::from_rect(&e1.rect),
                )
            }
            (Self::Projectile(e1), Self::Enemy(e2)) | (Self::Enemy(e2), Self::Projectile(e1)) => {
                let circle = Circle::from_rect(&e2.rect);
                e1.rect.check_collision_circle_rec(circle.center, circle.radius)
            }
            (Self::Enemy(e1), Self::Enemy(e2)) => {
                let c1 = Circle::from_rect(&e1.rect);
                let c2 = Circle::from_rect(&e2.rect);
                check_collision_circles(c1.center, c1.radius, c2.center, c2.radius)
            }
            _ => false,
        }
    }

    pub fn on_collision(&mut self, other: &mut Entity, world: &mut crate::World) -> Vec<Event> {
        let mut events = vec![];
        match (self, other) {
            (Self::Player(_e1), Self::Enemy(_e2)) | (Self::Enemy(_e2), Self::Player(_e1)) => {
                events.push(Event::GameOver);
            }
            (Self::Projectile(e1), Self::Enemy(e2)) | (Self::Enemy(e2), Self::Projectile(e1)) => {
                events.push(Event::ScoreIncreased);
                events.push(Event::NumberOfAsteroidsIncreased);
                events.push(Event::ResetAsteroid(e2.id));
                events.push(Event::EntityDestroyed(e1.id));
            }
            (Self::Enemy(e1), Self::Enemy(e2)) => {
                let (v1, v2) = crate::elastic_collision_1d(e1.mass, e1.velocity.y, e2.mass, e2.velocity.y);
                e1.velocity = Vector2::new(0., v1);
                e2.velocity = Vector2::new(0., v2);
            }
            _ => (),               
        };
        events
    }
    
    pub fn update(&mut self, rl: &mut RaylibHandle, delta_time: f32) {
        match self {
            Self::Player(e) => {
                // Handle horizontal movement (A/D or Left/Right)
                if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
                    e.velocity.x += e.acceleration * delta_time;
                } 
                else if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
                    e.velocity.x -= e.acceleration * delta_time;
                }

                // Handle vertical movement (W/S or Up/Down)
                if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
                    e.velocity.y -= e.acceleration * delta_time;
                } 
                else if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
                    e.velocity.y += e.acceleration * delta_time;
                }

                // Clamp velocity to max speed (using vector length for diagonal movement)
                let velocity_magnitude = (e.velocity.x * e.velocity.x + e.velocity.y * e.velocity.y).sqrt();
                if velocity_magnitude > e.max_velocity {
                    e.velocity.x = (e.velocity.x / velocity_magnitude) * e.max_velocity;
                    e.velocity.y = (e.velocity.y / velocity_magnitude) * e.max_velocity;
                }

                // Apply friction (only if no input is pressed)
                let is_moving_horizontally = rl.is_key_down(KeyboardKey::KEY_LEFT) || 
                                             rl.is_key_down(KeyboardKey::KEY_RIGHT) || 
                                             rl.is_key_down(KeyboardKey::KEY_A) || 
                                             rl.is_key_down(KeyboardKey::KEY_D);
                let is_moving_vertically = rl.is_key_down(KeyboardKey::KEY_UP) || 
                                           rl.is_key_down(KeyboardKey::KEY_DOWN) || 
                                           rl.is_key_down(KeyboardKey::KEY_W) || 
                                           rl.is_key_down(KeyboardKey::KEY_S);

                if !is_moving_horizontally {
                    e.velocity.x *= (1.0 - e.friction * delta_time).max(0.0);
                }
                if !is_moving_vertically {
                    e.velocity.y *= (1.0 - e.friction * delta_time).max(0.0);
                }

                // Update position
                e.rect.x += e.velocity.x * delta_time;
                e.rect.y += e.velocity.y * delta_time;

                // Clamp position to screen bounds
                e.rect.x = e.rect.x.clamp(0.0, SCREEN_WIDTH!() - e.rect.width);
                e.rect.y = e.rect.y.clamp(0.0, SCREEN_HEIGHT!() - e.rect.height);
            }
            Self::Enemy(e) => {
                e.rect.x += e.velocity.x * delta_time;
                e.rect.y += e.velocity.y * delta_time;
                e.rotation = (e.rotation + e.rotation_velocity) % 360. * delta_time;
                // Reset e when it goes off screen
                if e.rect.y > SCREEN_HEIGHT!() {
                    e.reset();
                }
            }
            Self::Projectile(e) => {
                e.rect.y -= e.speed as f32 * delta_time;
            }
            Self::Star(e) => {
                e.pos.y += e.velocity * delta_time;
                if e.pos.y > SCREEN_HEIGHT!() {
                    e.pos.x = rand::rng().random_range(0.0..SCREEN_HEIGHT!());
                    e.pos.y = -1.;
                }
            }
            _ => ()
        }    
    }

    pub fn _is_camera_affected(&self) -> bool {
        matches!(self, Self::Player(_) | Self::Enemy(_) | Self::Projectile(_))
    }
    
    pub fn draw(&self, d: &mut RaylibDrawHandle, textures: &Textures) {
        match self {
            Self::Player(e) => {
                d.draw_texture_pro(
                    textures.get("player").unwrap(),
                    Rectangle::new(0., 0., 40., 40.),
                    Rectangle::new(e.rect.x, e.rect.y, e.rect.width, e.rect.height + 40.),
                    Vector2::zero(),
                    0., Color::WHITE
                );
                // let tri = Triangle::from_rect(&e.rect);
                // d.draw_triangle(tri.a, tri.b, tri.c, Color::GOLD);
            }
            Self::Enemy(e) => {
                d.draw_texture_pro(
                    textures.get("asteroid").unwrap(),
                    Rectangle::new(0., 0., 23., 23.),
                    Rectangle::new(e.rect.x + e.rect.width / 2., e.rect.y + e.rect.height / 2., e.rect.width, e.rect.height),
                    Vector2::new(e.rect.width / 2., e.rect.height / 2.),
                    e.rotation, e.color
                );
                // d.draw_rectangle_rec(e.rect, e.color);
            }
            Self::Projectile(e) => {
                d.draw_rectangle_rec(e.rect, e.color);
            }
            Self::Star(e) => {
                d.draw_pixel_v(e.pos, e.color);
            }
            _ => ()
        }
    }
}


#[derive(Debug)]
pub struct Player {
    pub rect: Rectangle,
    acceleration: f32,
    friction: f32,
    max_velocity: f32,
    velocity: Vector2,
}

impl Player {
    pub fn new() -> Self {
        let width = 60.;
        let height = 60.;
        Self{
            rect: Rectangle::new(
                SCREEN_WIDTH!()/2. - width/2.,
                SCREEN_HEIGHT!() - height - 50.,
                width, height),
            acceleration: 1000.,
            friction: 10.,
            max_velocity: 500.,
            velocity: Vector2::new(0., 0.),
        }
    }
}


#[derive(Debug)]
pub struct Lazer {
    pub id: EntityId,
    pub rect: Rectangle,
    color: Color,
    speed: i32,
}

impl Lazer {
    pub fn new(id: EntityId, x: f32, y: f32) -> Self {
        Self {
            id,
            rect: Rectangle::new(x, y - 45., 2., 45.),
            color: Color::RED,
            speed: 1100,
        }
    }
}


#[derive(Debug)]
pub struct Star {
    pos: Vector2,
    velocity: f32,
    color: Color,
}

impl Star {
    pub fn new(velocity: f32) -> Self {
        let mut color = Color::RAYWHITE;
        color.a = rand::rng().random_range(100..255);
        Self {
            pos: Vector2::new(
                rand::rng().random_range(0.0..SCREEN_WIDTH!()),
                rand::rng().random_range(0.0..SCREEN_HEIGHT!())
            ),
            velocity,
            color
        }
    }
}


#[derive(Debug)]
pub struct Asteroid {
    pub id: EntityId,
    pub rect: Rectangle,
    velocity: Vector2,
    mass: f32,
    rotation: f32,
    rotation_velocity: f32,
    color: Color,
}

impl Asteroid {
    pub fn new(id: EntityId) -> Asteroid {
        let x = rand::rng().random_range(0.0..SCREEN_WIDTH!());
        let y = rand::rng().random_range(-SCREEN_HEIGHT!()..0.0);
        let size = rand::rng().random_range(10.0..40.);
        let width = size; 
        let height = size;
        Self {
            id,
            rect: Rectangle::new(x, y, width, height),
            velocity: Vector2::new(0., rand::rng().random_range(200.0..300.)),
            mass: 100. / 40. * size * 100.,
            rotation: 0.,
            rotation_velocity: rand::rng().random_range(-50.0..50.0),
            color: Color {
                r: rand::rng().random_range(200..255),
                g: rand::rng().random_range(235..255),
                b: rand::rng().random_range(245..255),
                a: 255
            }
        }
    }

    pub fn reset(&mut self) {
        let x = rand::rng().random_range(0.0..SCREEN_WIDTH!());
        let y = rand::rng().random_range(-SCREEN_HEIGHT!()..0.0);
        let size = rand::rng().random_range(10.0..40.);
        let width = size; 
        let height = size;
        *self = Self {
            id: self.id,
            rect: Rectangle::new(x, y, width, height),
            velocity: Vector2::new(0., rand::rng().random_range(200.0..300.)),
            mass: 100. / 40. * size * 100.,
            rotation: 0.,
            rotation_velocity: rand::rng().random_range(-50.0..50.0),
            color: Color {
                r: rand::rng().random_range(200..255),
                g: rand::rng().random_range(235..255),
                b: rand::rng().random_range(245..255),
                a: 255
            }
        };
    }
}
