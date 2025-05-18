use raylib::prelude::*;

#[derive(Debug)]
pub enum CollisionType {
    Circle,
    Rectangle,
    Triangle,
}

struct Circle {
    center: Vector2,
    radius: f32,    
}

struct Triangle {
    a: Vector2,
    b: Vector2,
    c: Vector2
}

impl Triangle {
    pub fn edges(&self) -> [(Vector2, Vector2); 3] {
        [
            (self.a, self.b),
            (self.b, self.c),
            (self.c, self.a),
        ]
    }
}

pub trait Collidable {
    fn rect(&self) -> &Rectangle;
    fn collides(&self, other: &dyn Collidable) -> bool {
        let rect1 = self.rect();
        let rect2 = other.rect();
        match (self.r#type(), other.r#type()) {
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
    fn r#type(&self) -> CollisionType;
}

// Helper function to calculate distance squared between points
fn distance_squared(a: Vector2, b: Vector2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

// Helper function to calculate distance from point to line segment
fn point_to_line_segment_distance(p: Vector2, a: Vector2, b: Vector2) -> f32 {
    let l2 = distance_squared(a, b);
    if l2 == 0.0 {
        return distance_squared(p, a).sqrt();
    }
    
    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    
    let projection = Vector2 {
        x: a.x + t * (b.x - a.x),
        y: a.y + t * (b.y - a.y),
    };
    
    distance_squared(p, projection).sqrt()
}

// Check if point is inside triangle (using barycentric coordinates)
fn point_in_triangle(p: Vector2, tri: &Triangle) -> bool {
    fn sign(a: Vector2, b: Vector2, c: Vector2) -> f32 {
        (a.x - c.x) * (b.y - c.y) - (b.x - c.x) * (a.y - c.y)
    }
    
    let d1 = sign(p, tri.a, tri.b);
    let d2 = sign(p, tri.b, tri.c);
    let d3 = sign(p, tri.c, tri.a);
    
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    
    !(has_neg && has_pos)
}

// Helper function to project a polygon onto an axis
fn project(poly: &[Vector2], axis: Vector2) -> (f32, f32) {
    let mut min = axis.x * poly[0].x + axis.y * poly[0].y;
    let mut max = min;
    
    for p in poly.iter().skip(1) {
        let proj = axis.x * p.x + axis.y * p.y;
        if proj < min {
            min = proj;
        }
        if proj > max {
            max = proj;
        }
    }
    
    (min, max)
}

// Check if two projections overlap
fn overlaps(a: (f32, f32), b: (f32, f32)) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}

// Calculate perpendicular axis
fn perpendicular(p1: Vector2, p2: Vector2) -> Vector2 {
    let edge = Vector2 {
        x: p2.x - p1.x,
        y: p2.y - p1.y,
    };
    Vector2 {
        x: -edge.y,
        y: edge.x,
    }
}

fn rect_to_poly(rect: &Rectangle) -> [Vector2; 4] {
    [
        Vector2 {
            x: rect.x,
            y: rect.y,
        },
        Vector2 {
            x: rect.x + rect.width,
            y: rect.y,
        },
        Vector2 {
            x: rect.x + rect.width,
            y: rect.y + rect.height,
        },
        Vector2 {
            x: rect.x,
            y: rect.y + rect.height,
        },
    ]
}

fn check_collision_rect_triangle(rect: &Rectangle, tri: &Triangle) -> bool {
    let rect_poly = rect_to_poly(rect);
    let tri_poly = [tri.a, tri.b, tri.c];
    
    // Check rectangle axes
    let rect_edges = [
        (rect_poly[0], rect_poly[1]),
        (rect_poly[1], rect_poly[2]),
    ];
    
    for edge in &rect_edges {
        let axis = perpendicular(edge.0, edge.1);
        let axis_normalized = {
            let len = (axis.x * axis.x + axis.y * axis.y).sqrt();
            Vector2 {
                x: axis.x / len,
                y: axis.y / len,
            }
        };
        
        let rect_proj = project(&rect_poly, axis_normalized);
        let tri_proj = project(&tri_poly, axis_normalized);
        
        if !overlaps(rect_proj, tri_proj) {
            return false;
        }
    }
    
    // Check triangle axes
    for edge in tri.edges() {
        let axis = perpendicular(edge.0, edge.1);
        let axis_normalized = {
            let len = (axis.x * axis.x + axis.y * axis.y).sqrt();
            Vector2 {
                x: axis.x / len,
                y: axis.y / len,
            }
        };
        
        let rect_proj = project(&rect_poly, axis_normalized);
        let tri_proj = project(&tri_poly, axis_normalized);
        
        if !overlaps(rect_proj, tri_proj) {
            return false;
        }
    }
    
    true
}

fn check_collision_circle_triangle(circle: &Circle, tri: &Triangle) -> bool {
    // Check if circle center is inside the triangle
    if point_in_triangle(circle.center, tri) {
        return true;
    }
    
    // Check distance to each edge
    for edge in tri.edges() {
        let distance = point_to_line_segment_distance(circle.center, edge.0, edge.1);
        if distance <= circle.radius {
            return true;
        }
    }
    
    // Check distance to each vertex
    let radius_sq = circle.radius * circle.radius;
    if distance_squared(circle.center, tri.a) <= radius_sq ||
       distance_squared(circle.center, tri.b) <= radius_sq ||
       distance_squared(circle.center, tri.c) <= radius_sq {
        return true;
    }
    
    false
}

pub fn elastic_collision_1d(m1: f32, v1: f32, m2: f32, v2: f32) -> (f32, f32) {
    if m1 == m2 {
        return (v2, v1);
    }
    let total_mass = m1 + m2;
    let v1_final = ((m1 - m2) * v1 + 2.0 * m2 * v2) / total_mass;
    let v2_final = ((m2 - m1) * v2 + 2.0 * m1 * v1) / total_mass;
    (v1_final, v2_final)
}

#[allow(clippy::too_many_arguments, dead_code)]
fn colliding_circles_1d(
    m1: f32, v1: f32, r1: f32, ω1: f32,
    m2: f32, v2: f32, r2: f32, ω2: f32,
    elasticity: f32
) -> (f32, f32, f32, f32) {
    // Moments of inertia (assuming solid spheres, k=2/5)
    let i1 = 0.4 * m1 * r1 * r1;
    let i2 = 0.4 * m2 * r2 * r2;
    
    // Effective masses accounting for rotation
    let m_eff1 = 1.0 / (1.0/m1 + r1*r1/i1);
    let m_eff2 = 1.0 / (1.0/m2 + r2*r2/i2);
    
    // Relative velocity at contact point
    let v_rel = (v1 - r1 * ω1) - (v2 + r2 * ω2);
    
    // Impulse magnitude
    let j = -(1.0 + elasticity) * v_rel / (1.0/m_eff1 + 1.0/m_eff2);
    
    // New linear velocities
    let v1_new = v1 + j / m1;
    let v2_new = v2 - j / m2;
    
    // New angular velocities
    let ω1_new = ω1 - j * r1 / i1;
    let ω2_new = ω2 + j * r2 / i2;
    
    (v1_new, ω1_new, v2_new, ω2_new)
}
