use bebytes::BeBytes;
use test_case::test_case;

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct VecLengthFromFieldName {
    pre_tail: u8,
    #[FromField(pre_tail)]
    tail: Vec<u8>,
    post_tail: u8,
}

#[test_case(VecLengthFromFieldName { pre_tail: 2, tail: vec![2, 3], post_tail: 4 }, 4; "test vec length from field name")]
fn test_vec_length_from_field_name(input: VecLengthFromFieldName, len: usize) {
    let bytes = input.clone().to_be_bytes();
    let (vec_length_from_field_name, written_len) =
        VecLengthFromFieldName::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(vec_length_from_field_name, input);
    assert_eq!(len, written_len);
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct PaddingVecWithVecLengthFromFieldName {
    innocent: u8,
    mid_tail: VecLengthFromFieldName,
    real_tail: Vec<u8>,
}

#[test_case(PaddingVecWithVecLengthFromFieldName { innocent: 1, mid_tail: VecLengthFromFieldName { pre_tail: 2, tail: vec![2, 3], post_tail: 4 }, real_tail: vec![5, 6] }, 7; "test padding vec with vec length from field name")]
fn test_padding_vec_with_vec_length_from_field_name(
    input: PaddingVecWithVecLengthFromFieldName,
    len: usize,
) {
    let bytes = input.clone().to_be_bytes();
    let (padding_vec_with_vec_length_from_field_name, written_len) =
        PaddingVecWithVecLengthFromFieldName::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(padding_vec_with_vec_length_from_field_name, input);
    assert_eq!(len, written_len);
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct WithSizeAttribute {
    #[With(size(3))]
    with_size: Vec<u8>,
    fourth: u8,
}

#[test_case(WithSizeAttribute { with_size: vec![1, 2, 3], fourth: 4 }, 4; "test with size attribute")]
fn test_with_size_attribute(input: WithSizeAttribute, len: usize) {
    let bytes = input.clone().to_be_bytes();
    let (with_size_attribute, written_len) = WithSizeAttribute::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(with_size_attribute, input);
    assert_eq!(len, written_len);
}

#[test]
fn test_vector_as_last_field() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct LastFieldVec {
        header: u8,
        data: Vec<u8>,
    }

    let test = LastFieldVec {
        header: 42,
        data: vec![1, 2, 3, 4, 5],
    };

    let bytes = test.to_be_bytes();
    assert_eq!(bytes[0], 42);
    assert_eq!(&bytes[1..], &[1, 2, 3, 4, 5]);

    let (result, _) = LastFieldVec::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(result, test);
}
