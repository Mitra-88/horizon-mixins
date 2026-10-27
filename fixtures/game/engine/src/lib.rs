pub mod entity;
pub mod calls;
#[path = "replacement.rs"]
pub mod overwrite;
pub mod inline {
    pub mod leaf;
    pub fn identity(value: i32) -> i32 { value }
}
