use arbitrary::Arbitrary;
use marsupial::{Hasher, SecurityLevel};

#[derive(Arbitrary)]
pub struct Input<'a> {
    input: &'a [u8],
    customization: &'a [u8],
    // this is necessary to keep allocations from being too large
    output_size: u16,
}

pub fn exercise_hasher<N>(data: Input<'_>)
where
    N: SecurityLevel,
{
    let hash = marsupial::hash::<N>(data.input);

    let mut hasher = Hasher::<N>::new();
    hasher.update(&data.input[..data.input.len() / 2]);
    hasher.update(&data.input[data.input.len() / 2..]);
    let hash2 = hasher.finalize();
    assert_eq!(hash, hash2);

    let mut hasher2 = Hasher::<N>::new();
    hasher2.update(data.input);
    let mut reader = hasher2.finalize_xof();
    let mut output = vec![0; N::HASH_ARRAY_LENGTH * 4];
    reader.squeeze(&mut output);
    assert_eq!(
        &output[..N::HASH_ARRAY_LENGTH],
        <N::Hash as Into<Vec<u8>>>::into(hash2)
    );

    let mut hasher = Hasher::<N>::new();
    hasher.update(data.input);
    let mut output = vec![0; data.output_size as usize];
    hasher
        .finalize_custom_xof(data.customization)
        .squeeze(&mut output);

    let mut hasher2 = Hasher::<N>::new();
    hasher2.update(&data.input[..data.input.len() / 2]);
    hasher2.update(&data.input[data.input.len() / 2..]);
    let mut output2 = vec![0; data.output_size as usize];
    hasher2
        .finalize_custom_xof(data.customization)
        .squeeze(&mut output2);

    assert_eq!(output, output2);
}
