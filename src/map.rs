use crate::{prelude::*, SCREEN_SIZE, Z_BASE_FLOOR, Z_BASE_OBJECTS};

pub fn get_floor_z(y: f32) -> f32 {
    Z_BASE_FLOOR - y / SCREEN_SIZE.1
}

pub fn get_object_z(y: f32) -> f32 {
    Z_BASE_OBJECTS - y / SCREEN_SIZE.1
}

pub fn as_object_vec3(vec: Vec2) -> Vec3 {
    vec.extend(get_object_z(vec.y))
}
