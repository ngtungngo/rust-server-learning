struct User {
    name: String,
}

impl User {
    fn greet(&self) {
        println!("Hallo, {}!", self.name);
    }

    fn rename(&mut self, new_name: &str) {
        self.name = String::from(new_name);
    }
}

fn main() {
    let mut user = User {
        name: String::from("Tung"),
    };

    user.greet();

    user.rename("Rust-Lerner");

    user.greet();
}