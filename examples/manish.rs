use serde::{Serialize, Deserialize};
use heapless::Vec;

#[derive(Serialize, Deserialize, Debug)]
struct Example {
    data: [u8; 12],
}

fn main() {
    let x: Vec<u8, 64> = postcard::to_vec(&Example{ data: [42; 12] }).unwrap();

    let y: Example = postcard::from_bytes(&x).unwrap();

    println!("{:?}", y);
}
