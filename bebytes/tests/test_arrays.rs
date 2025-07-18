use bebytes::BeBytes;
use test_case::test_case;

#[derive(BeBytes, Copy, Clone, Eq, PartialEq, Debug)]
pub struct ClientSetupResponse {
    pub mode: Modes,
    pub key_id: [u8; 1],
    pub token: [u8; 1],
    pub client_iv: [u8; 1],
}

#[derive(BeBytes, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Modes {
    pub bits: u8,
}

#[test_case(ClientSetupResponse { mode: Modes { bits: 1 }, key_id: [0; 1], token: [0; 1], client_iv: [0; 1] }, 4; "test arrays length")]
fn test_arrays(input: ClientSetupResponse, len: usize) {
    let bytes = input.clone().to_be_bytes();
    let (client_setup_response, written_len) =
        ClientSetupResponse::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(client_setup_response, input);
    assert_eq!(len, written_len);
}
