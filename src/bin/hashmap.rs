#[macro_use]
extern crate std;

use std::collections::HashMap;

macro_rules! hash_map {
    ($($key:expr => $val:expr),*) => {
        {
        let mut map = HashMap::new();
        $(
            map.insert($key,$val);
        )*
        map
        }
    };
}

fn main(){
    let map = hash_map!{
        "one"=>1,
        "two"=>2,
        "three"=>3
    };
    println!("{:?}",map);
}