use empa::abi;

#[derive(abi::Sized, Clone, Copy, PartialEq, Debug)]
#[repr(C)]
struct A {
    a0: u32,
    a1: u32,
}

#[derive(abi::Sized, Clone, Copy, PartialEq, Debug)]
#[repr(C)]
struct B {
    b0: u32,
    b1: [A; 16]
}

fn main() {}
