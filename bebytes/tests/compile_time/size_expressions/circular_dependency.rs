// This test verifies that circular dependencies in size expressions
// are detected and produce a compile-time error.

use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
extern crate alloc;

#[derive(BeBytes, Debug, PartialEq)]
struct CircularDependency {
    // Error: field1 depends on field2, which depends on field1
    #[bebytes(size = "field2")]
    field1: Vec<u8>,
    #[bebytes(size = "field1")]
    field2: Vec<u8>,
}

fn main() {}