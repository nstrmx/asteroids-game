use rand::*;
use raylib::prelude::*;

use crate::{SCREEN_WIDTH, SCREEN_HEIGHT, Textures, Collidable};


pub type EntityId = usize;

#[derive(Debug)]
pub enum EntityType {
    Player,
    Enemy,
    Projectile,
    Indestructible,
}

pub trait Entity {
    fn entity_type(&self) -> &EntityType;
    fn update(&mut self, rl: &mut RaylibHandle);
    fn draw(&self, d: &mut RaylibDrawHandle, textures: &Textures);
    fn as_collidable(&self) -> Option<&dyn Collidable> {
        None
    }
    fn as_movable(&self) -> Option<&dyn Movable> {
        None
    }
    fn as_movable_mut(&mut self) -> Option<&mut dyn Movable> {
        None
    }
}

pub trait Movable {
    fn vel(&self) -> &Vector2;
    fn vel_mut(&mut self) -> &mut Vector2;   
    fn mass(&self) -> &f32;
    fn rot(&self) -> &f32;
    fn rot_mut(&mut self) -> &mut f32;
    fn rot_vel(&self) -> &f32;
    fn rot_vel_mut(&mut self) -> &mut f32;
}

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
                SCREEN_WIDTH / 2. - width / 2.,
                SCREEN_HEIGHT - height - 50.,
                width, height),
            acceleration: 0.3,
            friction: 0.15,
            max_velocity: 5.,
            velocity: Vector2::new(0., 0.),
        }
    }
}

impl Entity for Player {
    fn entity_type(&self) -> &EntityType {
        &EntityType::Player
    }

    fn as_collidable(&self) -> Option<&dyn Collidable> {
        Some(self)
    }

    fn update(&mut self, rl: &mut RaylibHandle) {
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) | rl.is_key_down(KeyboardKey::KEY_D) {
            self.velocity.x = if self.velocity.x > self.max_velocity { self.max_velocity } else { self.velocity.x + self.acceleration };
        }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) | rl.is_key_down(KeyboardKey::KEY_A) {
            self.velocity.x = if self.velocity.x < -self.max_velocity { -self.max_velocity } else { self.velocity.x - self.acceleration };
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) | rl.is_key_down(KeyboardKey::KEY_W) {
            self.velocity.y = if self.velocity.y < -self.max_velocity { -self.max_velocity } else { self.velocity.y - self.acceleration };
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) | rl.is_key_down(KeyboardKey::KEY_S) {
            self.velocity.y = if self.velocity.y > self.max_velocity { self.max_velocity } else { self.velocity.y + self.acceleration };
        }

        self.rect.x += self.velocity.x;
        self.rect.y += self.velocity.y;

        if self.rect.x > SCREEN_WIDTH - self.rect.width {
            self.rect.x = SCREEN_WIDTH - self.rect.width;
        } else if self.rect.x < 0. {
            self.rect.x = 0.;
        }
        if self.rect.y > SCREEN_HEIGHT - self.rect.height {
            self.rect.y = SCREEN_HEIGHT - self.rect.height;
        } else if self.rect.y < 0. {
            self.rect.y = 0.;
        }

        if self.velocity.x > 0. {
            self.velocity.x -= f32::max(self.friction, -self.velocity.x);
        } else if self.velocity.x < 0. {
            self.velocity.x += f32::min(self.friction, -self.velocity.x);
        }
        if self.velocity.y > 0. {
            self.velocity.y -= f32::max(self.friction, -self.velocity.y);
        } else if self.velocity.y < 0. {
            self.velocity.y += f32::min(self.friction, -self.velocity.y);
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, textures: &Textures) {
        d.draw_texture_pro(
            textures.get("player").unwrap(),
            Rectangle::new(0., 0., 40., 40.),
            Rectangle::new(self.rect.x, self.rect.y, self.rect.width, self.rect.height + 40.),
            Vector2::zero(),
            0., Color::WHITE
        );
        // let rect1 = self.rect;
        // d.draw_triangle(
        //     Vector2::new(rect1.x, rect1.y+rect1.height),
        //     Vector2::new(rect1.x+rect1.width, rect1.y+rect1.height),
        //     Vector2::new(rect1.x+rect1.width/2., rect1.y),
        //     Color::GOLD
        // );
    }
}

pub struct Lazer {
    pub rect: Rectangle,
    color: Color,
    speed: i32,
}

impl Lazer {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            rect: Rectangle::new(x, y - 45., 2., 45.),
            color: Color::RED,
            speed: 11,
        }
    }
}

impl Entity for Lazer {
    fn entity_type(&self) -> &EntityType {
        &EntityType::Projectile    
    }
    
    fn as_collidable(&self) -> Option<&dyn Collidable> {
        Some(self)
    }

    fn update(&mut self, _rl: &mut RaylibHandle) {
        self.rect.y -= self.speed as f32;
    }

    fn draw(&self, d: &mut RaylibDrawHandle, _textures: &Textures) {
        d.draw_rectangle_rec(self.rect, self.color);
    } 
}

pub struct Barrier {
    pub rect: Rectangle,
}

impl Barrier {
    pub fn new(width: f32, height: f32) -> Self {
        Self{rect: Rectangle::new(0., -height, width, height) }
    }
}

impl Entity for Barrier {
    fn entity_type(&self) -> &EntityType {
        &EntityType::Indestructible        
    }

    fn update(&mut self, _rl: &mut RaylibHandle) {}

    fn draw(&self, _d: &mut RaylibDrawHandle, _: &Textures) {}

    fn as_collidable(&self) -> Option<&dyn Collidable> {
        Some(self)
    }
}

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
                rand::rng().random_range(0.0..SCREEN_WIDTH),
                rand::rng().random_range(0.0..SCREEN_HEIGHT)
            ),
            velocity,
            color
        }
    }
}

impl Entity for Star {
    fn entity_type(&self) -> &EntityType {
        &EntityType::Indestructible
    }

    fn update(&mut self, _rl: &mut RaylibHandle) {
        self.pos.y += self.velocity;
        if self.pos.y > SCREEN_HEIGHT {
            self.pos.x = rand::rng().random_range(0.0..SCREEN_WIDTH);
            self.pos.y = -1.;
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, _textures: &Textures) {
        d.draw_pixel_v(self.pos, self.color);
    }
}

pub struct Asteroid {
    pub rect: Rectangle,
    velocity: Vector2,
    mass: f32,
    rotation: f32,
    rotation_velocity: f32,
    color: Color,
}

impl Asteroid {
    pub fn new() -> Asteroid {
        let x = rand::rng().random_range(0.0..SCREEN_WIDTH);
        let y = rand::rng().random_range(-SCREEN_HEIGHT..0.0);
        let size = rand::rng().random_range(10.0..40.);
        let width = size; 
        let height = size;
        Self{
            rect: Rectangle::new(x, y, width, height),
            velocity: Vector2::new(0., rand::rng().random_range(2.0..5.)),
            mass: 200. / 40. * size,
            rotation: 0.,
            rotation_velocity: rand::rng().random_range(-5.0..5.0),
            color: Color {
                r: rand::rng().random_range(200..255),
                g: rand::rng().random_range(235..255),
                b: rand::rng().random_range(245..255),
                a: 255
            }
        }
    }
}

impl Entity for Asteroid {
    fn entity_type(&self) -> &EntityType {
        &EntityType::Enemy
    }

    fn as_collidable(&self) -> Option<&dyn Collidable> {
        Some(self)
    }

    fn as_movable(&self) -> Option<&dyn Movable> {
        Some(self)
    }

    fn as_movable_mut(&mut self) -> Option<&mut dyn Movable> {
        Some(self)
    }

    fn update(&mut self, _rl: &mut RaylibHandle) {
        self.rect.x += self.velocity.x;
        self.rect.y += self.velocity.y;
        self.rotation = (self.rotation + self.rotation_velocity) % 360.;
        // Reset self when it goes off screen
        if self.rect.y > SCREEN_HEIGHT {
            self.velocity.x = 0.;
            self.velocity.y = rand::rng().random_range(2.0..5.);
            self.rect.x = rand::rng().random_range(0.0..SCREEN_WIDTH-self.rect.width);
            self.rect.y = -self.rect.height - 10.;
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, textures: &Textures) {
        d.draw_texture_pro(
            textures.get("asteroid").unwrap(),
            Rectangle::new(0., 0., 23., 23.),
            Rectangle::new(self.rect.x + self.rect.width / 2., self.rect.y + self.rect.height / 2., self.rect.width, self.rect.height),
            Vector2::new(self.rect.width / 2., self.rect.height / 2.),
            self.rotation, self.color
        );
        // d.draw_rectangle_rec(self.rect, self.color);
    }
}

impl Movable for Asteroid {
    fn vel(&self) -> &Vector2 {
        &self.velocity
    }

    fn vel_mut(&mut self) -> &mut Vector2 {
        &mut self.velocity
    }

    fn mass(&self) -> &f32 {
        &self.mass
    }

    fn rot(&self) -> &f32 {
        &self.rotation
    }

    fn rot_mut(&mut self) -> &mut f32 {
        &mut self.rotation
    }

    fn rot_vel(&self) -> &f32 {
        &self.rotation_velocity
    }

    fn rot_vel_mut(&mut self) -> &mut f32 {
        &mut self.rotation_velocity
    }
}
