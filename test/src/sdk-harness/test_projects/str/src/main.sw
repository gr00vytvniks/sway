script;

use std::{ assert::assert, str_ops::* };

fn main() -> bool {
    let a: str[4] = "Fuel";
    let b: str[4] = "Fuel";
    let c: str[4] = "Fule";
    let d: str[4] = "Sway";
    let f: str[5] = "Chain";
    let e = a;

    assert(string_eq(a, b));

    true
}
