use std::collections::{BTreeMap, HashMap};
use raylib::prelude::*;

mod events;
mod entities;
mod collision;

use events::*;
use entities::*;
use collision::*;

const SCREEN_WIDTH: f32 = 1200.0;
const SCREEN_HEIGHT: f32 = 600.0;

type Textures = HashMap<&'static str, Texture2D>;


struct Context {
    id_count: EntityId,
    player_id: EntityId,
    entities: BTreeMap<EntityId, Box<dyn Entity>>,
    collidables: Vec<EntityId>,
    score: i32,
    enemy_max: u32,
    enemy_count: u32,
    over: bool,
}

impl Context {
    fn new_id(&mut self) -> EntityId {
        let id = self.id_count;
        self.id_count += 1;
        id
    }

    fn new_asteroid(&mut self) {
        let id = self.new_id();
        let asteroid = Asteroid::new();
        self.entities.insert(id, Box::new(asteroid));
        self.enemy_count += 1;
        self.collidables.push(id);
    }

    fn new_barrier(&mut self) {
        let id = self.new_id();
        let barrier = Barrier::new(SCREEN_WIDTH, 10.);
        self.entities.insert(id, Box::new(barrier));
        self.collidables.push(id);
    }

    fn new_star(&mut self, velocity: f32) {
        let id = self.new_id();
        let star = Star::new(velocity);
        self.entities.insert(id, Box::new(star));
    }
    
    fn new_player(&mut self) {
        let id = self.new_id();
        self.player_id = id;
        let player = Player::new();
        self.entities.insert(id, Box::new(player));
        self.collidables.push(id);
    }

    fn new_lazer(&mut self, x: f32, y: f32) {
        let id = self.new_id();
        let lazer = Lazer::new(x, y);
        self.entities.insert(id, Box::new(lazer));
        self.collidables.push(id);
    }
}

struct Game {
    rl: RaylibHandle,
    rt: RaylibThread,
    _event_bus: EventBus,
    context: Context,
    textures: Textures,
}

impl Game {
    fn new() -> Self {
        let (mut rl, rt) = raylib::init()
            .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
            .title("Asteroids")
            .vsync()
            .build();
        rl.set_target_fps(60);

        Self {
            rl, rt,
            _event_bus: EventBus::new(),
            context: Context {
                id_count: 0,
                player_id: 0,
                entities: BTreeMap::new(),
                collidables: vec![],
                score: 0,
                enemy_max: 10,
                enemy_count: 0,
                over: false,
            },
            textures: HashMap::new(),
         }    
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
        self.context.new_barrier();
        
        for _i in 0..5000 {
            self.context.new_star(0.05);
        }

        for _i in 0..1000 {
            self.context.new_star(0.1);
        }
         
        self.context.new_player();
        for _i in 0..self.context.enemy_max {
            self.context.new_asteroid();
        }
    }
    
    fn check_collisions(&mut self) {
        let mut collidables: Vec<EntityId> = Vec::with_capacity(self.context.collidables.len());
        std::mem::swap(&mut collidables, &mut self.context.collidables);

        let mut i = 0;
        'outer: while i < collidables.len() {
            let id1 = collidables[i];
            let mut destroyed = false;
            let mut j = i + 1;
            while j < collidables.len() {
                let id2 = collidables[j];
                let entity1_opt = self.context.entities.get(&id1);
                if entity1_opt.is_none() {
                    i += 1;
                    continue 'outer;
                }
                let entity1 = entity1_opt.unwrap();
                
                let entity2_opt = self.context.entities.get(&id2);
                if entity2_opt.is_none() {
                    j += 1;
                    continue;
                }
                let entity2 = entity2_opt.unwrap();

                let col1 = entity1.as_collidable().unwrap();
                let col2 = entity2.as_collidable().unwrap(); 
        
                if col1.collides(col2) {
                    match (entity1.r#type(), entity2.r#type()) {
                        (EntityType::Player, EntityType::Enemy)
                        | (EntityType::Enemy, EntityType::Player) => {
                            destroyed = true;
                            self.context.over = true;
                            break;
                        }
                        (EntityType::Projectile, EntityType::Enemy) => {
                            self.context.entities.remove(&id1);
                            *self.context.entities.get_mut(&id2).unwrap() = Box::new(Asteroid::new());
                            self.context.score += 100;
                            self.context.new_asteroid();
                        }
                        (EntityType::Enemy, EntityType::Projectile) => {
                            self.context.entities.remove(&id2);
                            *self.context.entities.get_mut(&id1).unwrap() = Box::new(Asteroid::new());
                            self.context.score += 100;
                            self.context.new_asteroid();
                        }
                        (EntityType::Projectile, EntityType::Projectile) => {
                            destroyed = true;
                            self.context.entities.remove(&id1);
                            self.context.entities.remove(&id2);
                        }
                        (EntityType::Indestructible, EntityType::Projectile) => {
                            self.context.entities.remove(&id2);
                        }
                        (EntityType::Enemy, EntityType::Enemy) => {
                            let mov1 = entity1.as_movable().unwrap();
                            let mov2 = entity2.as_movable().unwrap();
                            
                            let m1 = *mov1.mass();
                            let v1 = *mov1.vel();
                            
                            let m2 = *mov2.mass();
                            let v2 = *mov2.vel();

                            let (v1, v2) = elastic_collision_1d(m1, v1.y, m2, v2.y);

                            *self.context.entities.get_mut(&id1).unwrap().as_movable_mut().unwrap().vel_mut() = Vector2::new(0., v1);
                            *self.context.entities.get_mut(&id2).unwrap().as_movable_mut().unwrap().vel_mut() = Vector2::new(0., v2);
                        }
                        _ => (),               
                    }
                }
                j += 1;
            }
            if !destroyed {
                self.context.collidables.push(id1);
            }
            i += 1;
        }
    }

    fn update(&mut self) {
        if self.context.over {
            return;
        }
        
        if self.rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            let player_rect = *self.context.entities.get(&self.context.player_id).unwrap().as_collidable().unwrap().rect();
            self.context.new_lazer(player_rect.x + player_rect.width / 3., player_rect.y);
            self.context.new_lazer(player_rect.x + player_rect.width / 2., player_rect.y);
            self.context.new_lazer(player_rect.x + player_rect.width - player_rect.width / 3., player_rect.y);
        }
        
        self.check_collisions();

        println!("enemy count: {}", self.context.enemy_count);

        for (_id, entity) in self.context.entities.iter_mut() {
            entity.update(&mut self.rl);
        }    
    }

    fn draw(&mut self) {
        let mut d = self.rl.begin_drawing(&self.rt);
        d.clear_background(Color::BLACK);

        for (_id, entity) in self.context.entities.iter() {
            entity.draw(&mut d, &self.textures);
        }

        // Draw UI
        d.draw_text(&format!("score: {}", self.context.score), 35, 10, 20, Color::WHITE);

        if self.context.over {
            d.draw_text(
                &format!("YOUR SCORE: {}", self.context.score), 
                (SCREEN_WIDTH/2. - 180.) as i32, 
                (SCREEN_HEIGHT/2.) as i32, 
                40, Color::WHITE
            );
            d.draw_text(
                "Press any key to exit", 
                (SCREEN_WIDTH/2. - 180.) as i32, 
                (SCREEN_HEIGHT/2. + 40.) as i32, 
                30, Color::WHITE
            );
        }
    }

    fn run(&mut self) {
        self.setup();

        while !self.rl.window_should_close() {
            if self.context.over && self.rl.is_key_down(KeyboardKey::KEY_ESCAPE) {
                break;
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
