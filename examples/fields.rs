use bevy::prelude::*;
use mod_plugins::macros::*;

fn main() {
    App::new()
        .add_plugins(FieldTest { name: "Steve", score: 23 })
        .run();
}

#[plugin]
mod field_test {
    #[field] pub type name = &'static str;
    #[field] type score = u32;

    #[build]
    fn field_test(&self, _app: &mut App) {
        println!("Name {:?} with score {:?}", self.name, self.score);
    }
}