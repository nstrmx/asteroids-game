pub trait Transformable {
    fn vel(&self) -> &Vector2;
    fn vel_mut(&mut self) -> &mut Vector2;   
    fn mass(&self) -> &f32;
    fn rot(&self) -> &f32;
    fn rot_mut(&mut self) -> &mut f32;
    fn rot_vel(&self) -> &f32;
    fn rot_vel_mut(&mut self) -> &mut f32;
}
