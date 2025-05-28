use std::collections::HashMap;
use raylib::prelude::*;

mod events;
mod entities;
mod collision;

use events::*;
use entities::*;
use collision::*;


static mut SCREEN_WIDTH: i32 = 1200;
static mut SCREEN_HEIGHT: i32 = 800;

#[macro_export]
macro_rules! SCREEN_WIDTH {
    () => { unsafe{ SCREEN_WIDTH as f32 }};
    ($w:expr) => { unsafe{ SCREEN_WIDTH = $w; }}
}
#[macro_export]
macro_rules! SCREEN_HEIGHT {
    () => { unsafe{ SCREEN_HEIGHT as f32 }};
    ($h:expr) => { unsafe{ SCREEN_HEIGHT = $h; }}
}

type Textures = HashMap<&'static str, Texture2D>;


struct World {
    id_count: EntityId,
    player_id: EntityId,
    entities: HashMap<EntityId, Entity>,
    drawables: Vec<EntityId>,
    collidables: Vec<EntityId>,
    enemy_max: u32,
    enemy_count: u32,
}

impl World {
    fn new_id(&mut self) -> EntityId {
        let id = self.id_count;
        self.id_count += 1;
        id
    }

    fn new_asteroid(&mut self) {
        let id = self.new_id();
        let asteroid = Asteroid::new(id);
        let entity = Entity::Enemy(Box::new(asteroid));
        self.entities.insert(id, entity);
        self.enemy_count += 1;
        self.collidables.push(id);
        self.drawables.push(id);
    }

    fn new_star(&mut self, velocity: f32) {
        let id = self.new_id();
        let star = Star::new(velocity);
        let entity = Entity::Star(Box::new(star));
        self.entities.insert(id, entity);
        self.drawables.push(id);
    }
    
    fn new_player(&mut self) {
        let id = self.new_id();
        self.player_id = id;
        let player = Player::new();
        let entity = Entity::Player(Box::new(player));
        self.entities.insert(id, entity);
        self.collidables.push(id);
        self.drawables.push(id);
    }

    fn new_lazer(&mut self, x: f32, y: f32) {
        let id = self.new_id();
        let lazer = Lazer::new(id, x, y);
        let entity = Entity::Projectile(Box::new(lazer));
        self.entities.insert(id, entity);
        self.collidables.push(id);
        self.drawables.push(id);
    }
}

struct Game {
    rl: RaylibHandle,
    rt: RaylibThread,
    event_bus: EventBus,
    world: World,
    textures: Textures,
    camera: Camera2D,
    paused: bool,
    score: usize,
    over: bool,
}

impl Game {
    fn new() -> Self {
        let (mut rl, rt) = raylib::init()
            .title("Asteroids")
            .size(SCREEN_WIDTH!() as i32, SCREEN_HEIGHT!() as i32)
            .vsync()
            .build();
        rl.set_target_fps(60);

        Self {
            rl, rt,
            event_bus: EventBus::new(),
            world: World {
                id_count: 0,
                player_id: 0,
                entities: HashMap::new(),
                drawables: vec![],
                collidables: vec![],
                enemy_max: 10,
                enemy_count: 0,
            },
            textures: HashMap::new(),
            camera: Camera2D::default(),
            score: 0,
            paused: false,
            over: false,
        }    
    }

    fn toggle_fullscreen(&mut self) {
        self.rl.toggle_fullscreen();
    }

    fn load_textures(&mut self) {
        let texture_list = vec![
            ("player", "assets/player.png"),
            ("asteroid", "assets/asteroid.png"),
        ];
        for (name, filename) in texture_list {
            let texture = self.rl.load_texture(&self.rt, filename).unwrap();
            self.textures.insert(name, texture);
        }
    }

    fn setup(&mut self) {
        self.load_textures();
        
        for _i in 0..5000 {
            self.world.new_star(0.5);
        }

        for _i in 0..1000 {
            self.world.new_star(1.);
        }
         
        self.world.new_player();

        if let Some(Entity::Player(player)) = self.world.entities.get(&self.world.player_id) {
            self.camera.target = Vector2::new(player.rect.x + 20.0, player.rect.y + 20.0);
            self.camera.offset = Vector2::new(SCREEN_WIDTH!()/2.0, SCREEN_HEIGHT!() - player.rect.height - 40.);
            self.camera.rotation = 0.0;
            self.camera.zoom = 0.5;    
        };
        
        for _i in 0..self.world.enemy_max {
            self.world.new_asteroid();
        }

        self.event_bus.subscribe(EventType::EntityDestroyed, Box::new(|e: &Event, ctx: &mut Game|{
            
        }));
    }
    
    fn check_collisions(&mut self) {
        let world_ptr = &mut self.world as *mut World;
        
        let mut new_asteroids_count = 0;
        let mut destroyed = vec![];
        let mut reset_asteroids = vec![];
        
        let mut i = 0;
        'outer: while i < self.world.collidables.len() {
            let id1 = self.world.collidables[i];
            let mut j = i + 1;
            while j < self.world.collidables.len() {
                let id2 = self.world.collidables[j];
                let e1 = if let Some(e) = self.world.entities.get_mut(&id1) {
                    let ptr = e as *mut Entity;
                    // SAFETY: Safe unless we do not cause entities' reallocation
                    unsafe { &mut *ptr }
                } else {
                    i += 1;
                    continue 'outer;
                };
                let e2 = if let Some(e) = self.world.entities.get_mut(&id2) {
                    let ptr = e as *mut Entity;
                    // SAFETY: Safe unless we do not cause entities' reallocation
                    unsafe { &mut *ptr }
                } else {
                    j += 1;
                    continue;
                };
                if e1.check_collision(e2) {
                    let world = unsafe{ &mut *world_ptr };
                    for event in e1.on_collision(e2, world) {
                        match event {
                            Event::EntityDestroyed(id) => {
                                destroyed.push(id);
                                if id == id1 {
                                    i += 1;
                                    continue 'outer;
                                } 
                            }
                            Event::NumberOfAsteroidsIncreased => {
                                new_asteroids_count += 1;                                
                            }
                            Event::ResetAsteroid(id) => {
                                reset_asteroids.push(id);
                            }
                            Event::ScoreIncreased => {
                                self.score += 100;
                            }
                            Event::GameOver => {
                                self.over = true;
                                return;
                            }
                        }
                    }
                }
                j += 1;
            }
            for id in reset_asteroids.drain(..) {
                if let Some(Entity::Enemy(e)) = self.world.entities.get_mut(&id) {
                    e.reset();
                }
            }
            i += 1;
        }
        for id in destroyed.drain(..) {
            self.world.entities.remove(&id);
        }
        for _ in 0..new_asteroids_count {
            self.world.new_asteroid();
        }
    }

    fn update(&mut self) {
        if self.rl.is_key_pressed(KeyboardKey::KEY_F) {
            if !self.paused {
                self.paused = true;                
            }

            self.toggle_fullscreen();
        }

        if self.over {
            return;
        }

        if self.rl.is_key_pressed(KeyboardKey::KEY_PAUSE) {
            self.paused = !self.paused;
        }

        if self.paused {
            return;
        }
        
        if self.rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            let opt = if let Some(Entity::Player(e)) = self.world.entities.get(&self.world.player_id) {
                Some(e.rect)
            } else { None };
            if let Some(rect) = opt {
                self.world.new_lazer(rect.x + rect.width / 3., rect.y);
                self.world.new_lazer(rect.x + rect.width / 2., rect.y);
                self.world.new_lazer(rect.x + rect.width - rect.width / 3., rect.y);
            }
        }
        
        self.check_collisions();

        let delta_time = self.rl.get_frame_time();
        for (_id, entity) in self.world.entities.iter_mut() {
            entity.update(&mut self.rl, delta_time);
        }    
    }

    fn draw(&mut self) {
        let mut d = self.rl.begin_drawing(&self.rt);
        d.clear_background(Color::BLACK);

        for (_, entity) in self.world.entities.iter() {
            entity.draw(&mut d, &self.textures);
        }

        // {
        //     let mut m = d.begin_mode2D(self.camera);

        //     for (_, entity) in self.world.entities.iter().filter(|(_, e)| e.is_camera_affected()) {
        //         entity.draw(&mut m, &self.textures);
        //     }
        // }

        // Draw UI
        d.draw_text(&format!("score: {}", self.score), 35, 10, 20, Color::WHITE);
        d.draw_fps(35, 30);

        if self.over {
            d.draw_text(
                &format!("YOUR SCORE: {}", self.score), 
                (SCREEN_WIDTH!()/2. - 180.) as i32, 
                (SCREEN_HEIGHT!()/2.) as i32, 
                40, Color::WHITE
            );
            d.draw_text(
                "Press any key to exit", 
                (SCREEN_WIDTH!()/2. - 180.) as i32, 
                (SCREEN_HEIGHT!()/2. + 40.) as i32, 
                30, Color::WHITE
            );
        } else if self.paused {
            d.draw_text(
                "GAME PAUSED", 
                (SCREEN_WIDTH!()/2. - 180.) as i32, 
                (SCREEN_HEIGHT!()/2.) as i32, 
                40, Color::WHITE
            );
            d.draw_text(
                "Press PAUSE to continue", 
                (SCREEN_WIDTH!()/2. - 180.) as i32, 
                (SCREEN_HEIGHT!()/2. + 40.) as i32, 
                30, Color::WHITE
            );
        }
    }

    fn run(&mut self) {
        self.setup();

        while !self.rl.window_should_close() {
            if self.over {
                if let Some(key) = self.rl.get_key_pressed() {
                    match key {
                        KeyboardKey::KEY_SPACE
                        | KeyboardKey::KEY_LEFT | KeyboardKey::KEY_RIGHT
                        | KeyboardKey::KEY_UP | KeyboardKey::KEY_DOWN
                        | KeyboardKey::KEY_A | KeyboardKey::KEY_D
                        | KeyboardKey::KEY_W | KeyboardKey::KEY_S => (),
                        _ => break,
                    }
                }
            }
            self.update();
            self.draw();
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.textures.clear();
    }
}

fn main() {
    Game::new().run();    
}
