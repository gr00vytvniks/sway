script; 

dep foo;

struct S<T> { }

impl<T> S<T> {
  fn f(self) -> u64 {
    5
  }
}

fn main() -> u64 {
  let a = S::<u64> { };
  let b = foo::baz::ExampleStruct::<u64, bool> { a_field: 5u64, b_field: true };
  return a.f();
}
