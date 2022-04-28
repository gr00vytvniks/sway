//! A collection of utils to facilitate working with the str[n] type.
library str_ops;


/// returns the boolean result of comparing 2 str[n] values.
pub fn string_eq<T, O>(str_a: T, str_b: O) -> bool {
    let size = size_of::<T>();
    let other_size = size_of::<O>();
    if size == other_size {
        // compare string contents
        asm(r1, r2: str_a, r3: str_b, r4: size) {
            meq r1 r2 r3 r4;
            r1: bool
        }
    } else {
        false
    }
}
