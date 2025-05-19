use raylib::prelude::*;
use crate::entities::*;
use crate::collision::*;

pub trait Collidable: Entity {
    fn rect(&self) -> &Rectangle;
    fn check_collision(&self, other: &dyn Collidable) -> bool {
        let rect1 = self.rect();
        let rect2 = other.rect();
        match (self.collision_type(), other.collision_type()) {
            (CollisionType::Circle, CollisionType::Circle) => {
                check_collision_circles(
                    Vector2::new(rect1.x + rect1.width/2., rect1.y + rect1.height/2.),
                    rect1.width/2.,
                    Vector2::new(rect2.x + rect2.width/2., rect2.y + rect2.height/2.),
                    rect2.width/2.,
                )
            }
            (CollisionType::Circle, CollisionType::Rectangle) => {
                rect2.check_collision_circle_rec(
                    Vector2::new(rect1.x + rect1.width/2., rect1.y + rect1.height/2.),
                    rect1.width/2.,
                )
            }
            (CollisionType::Circle, CollisionType::Triangle) => {
                check_collision_circle_triangle(
                    &Circle{
                        center: Vector2::new(rect1.x+rect1.width/2., rect1.y+rect1.height/2.),
                        radius: rect1.width/2.,
                    },
                    &Triangle{
                        a: Vector2::new(rect2.x, rect2.y+rect2.height),
                        b: Vector2::new(rect2.x+rect2.width, rect2.y+rect2.height),
                        c: Vector2::new(rect2.x+rect2.width/2., rect2.y)
                    }
                )
            }
            (CollisionType::Rectangle, CollisionType::Rectangle) => {
                rect1.check_collision_recs(rect2)   
            }
            (CollisionType::Rectangle, CollisionType::Circle) => {
                rect1.check_collision_circle_rec(
                    Vector2::new(rect2.x + rect2.width/2., rect2.y + rect2.height/2.),
                    rect2.width/2.,
                )
            }
            (CollisionType::Rectangle, CollisionType::Triangle) => {
               check_collision_rect_triangle(rect1, &Triangle {
                    a: Vector2::new(rect2.x, rect2.y+rect2.height),
                    b: Vector2::new(rect2.x+rect2.width, rect2.y+rect2.height),
                    c: Vector2::new(rect2.x+rect2.width/2., rect2.y)
                })
            }
            (CollisionType::Triangle, CollisionType::Circle) => {
                check_collision_circle_triangle(
                    &Circle{
                        center: Vector2::new(rect2.x+rect2.width/2., rect2.y+rect2.height/2.),
                        radius: rect2.width/2.,
                    },
                    &Triangle{
                        a: Vector2::new(rect1.x, rect1.y+rect1.height),
                        b: Vector2::new(rect1.x+rect1.width, rect1.y+rect1.height),
                        c: Vector2::new(rect1.x+rect1.width/2., rect1.y)
                    }
                )
            }
            (CollisionType::Triangle, CollisionType::Rectangle) => {
                check_collision_rect_triangle(rect2, &Triangle { 
                    a: Vector2::new(rect1.x, rect1.y+rect1.height),
                    b: Vector2::new(rect1.x+rect1.width, rect1.y+rect1.height),
                    c: Vector2::new(rect1.x+rect1.width/2., rect1.y)
                })
            }
            (CollisionType::Triangle, CollisionType::Triangle) => {
                unreachable!()
            }
        }
    }
    fn collision_type(&self) -> CollisionType;
}

impl Collidable for Player {
    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn collision_type(&self) -> CollisionType {
        CollisionType::Triangle
    }
}

impl Collidable for Lazer {
    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn collision_type(&self) -> CollisionType {
        CollisionType::Rectangle
    }
}

impl Collidable for Barrier {
    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn collision_type(&self) -> CollisionType {
        CollisionType::Rectangle
    }
}

impl Collidable for Asteroid {
    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn collision_type(&self) -> CollisionType {
        CollisionType::Circle
    }
}
