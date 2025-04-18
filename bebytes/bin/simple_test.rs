#![allow(clippy::assign_op_pattern)]

use bebytes::*;

fn main() {
    let for_tailing = ForTailingVector { tail_size: 3 };
    let for_tailing_meta = ForTailingVector::requires_external_sizes();
    println!("ForTailingVector: {:?}", for_tailing_meta);
    let with_tailing = WithTailingVec {
        pre_tail: 3,
        tail: vec![2, 3, 4],
        post_tail: 5,
    };
    let with_tailing_meta = WithTailingVec::requires_external_sizes();
    println!("WithTailing: {:?}", with_tailing_meta);
    let _parent = Parent {
        for_tailing,
        with_tailing: with_tailing,
    };
    let parent_meta = Parent::requires_external_sizes();
    println!("Parent: {:?}", parent_meta);
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct Parent {
    for_tailing: ForTailingVector,
    with_tailing: WithTailingVec,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct ForTailingVector {
    tail_size: u32,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithTailingVec {
    pre_tail: u8,
    #[FromField(pre_tail)]
    tail: Vec<u8>,
    post_tail: u8,
}
