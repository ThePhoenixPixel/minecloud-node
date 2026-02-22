use bx::network::address::Address;
use uuid::Uuid;

pub struct Node {
    name: String,
    uuid: Uuid,
    host: Address,
}

impl Node {
    fn new() {
        print!("noide etst");
    }
}
