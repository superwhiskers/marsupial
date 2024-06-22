use crate::{hash, Hasher};
use digest::{ExtendableOutput, Update, XofReader};
use tiny_keccak::{IntoXof, Xof};

#[test]
#[should_panic]
fn test_update_after_finalize_panics() {
    let mut hasher = Hasher::<128>::new();
    hasher.finalize();
    hasher.update(&[]);
}

#[test]
#[should_panic]
fn test_finalize_twice_panics() {
    let mut hasher = Hasher::<128>::new();
    hasher.finalize();
    hasher.finalize();
}

fn fill_pattern(buf: &mut [u8]) {
    // repeating the pattern 0x00, 0x01, 0x02, ..., 0xFA as many times as necessary
    for i in 0..buf.len() {
        buf[i] = (i % 251) as u8;
    }
}

fn kt256_hex(input: &[u8], customization: &[u8], num_output_bytes: usize) -> String {
    let mut hasher = Hasher::<256>::new();
    hasher.update(input);
    let mut output = vec![0; num_output_bytes];
    hasher
        .finalize_custom_xof(customization)
        .squeeze(&mut output);

    // check that doing the same hash in two steps gives the same answer
    let mut hasher2 = Hasher::<256>::new();
    hasher2.update(&input[..input.len() / 2]);
    hasher2.update(&input[input.len() / 2..]);
    let mut output2 = vec![0; num_output_bytes];
    hasher2
        .finalize_custom_xof(customization)
        .squeeze(&mut output2);
    assert_eq!(output, output2);

    // check that using the all-at-once function gives the same answer if possible
    if customization.is_empty() {
        let hash3 = hash::<256>(input);
        let compare_len = std::cmp::min(hash3.as_bytes().len(), num_output_bytes);
        assert_eq!(&hash3.as_bytes()[..compare_len], &output[..compare_len]);
    }

    hex::encode(output)
}

fn kt128_hex(input: &[u8], customization: &[u8], num_output_bytes: usize) -> String {
    let mut hasher = Hasher::<128>::new();
    hasher.update(input);
    let mut output = vec![0; num_output_bytes];
    hasher
        .finalize_custom_xof(customization)
        .squeeze(&mut output);

    // Also check that doing the same hash in two steps gives the same answer.
    let mut hasher2 = Hasher::<128>::new();
    hasher2.update(&input[..input.len() / 2]);
    hasher2.update(&input[input.len() / 2..]);
    let mut output2 = vec![0; num_output_bytes];
    hasher2
        .finalize_custom_xof(customization)
        .squeeze(&mut output2);
    assert_eq!(output, output2);

    // Check that the all-at-once function gives the same answer too.
    if customization.is_empty() {
        let hash3 = hash::<128>(input);
        let compare_len = std::cmp::min(hash3.as_bytes().len(), num_output_bytes);
        assert_eq!(&hash3.as_bytes()[..compare_len], &output[..compare_len]);
    }

    // Check that the `k12` crate gives the same answer too.
    let mut k12_state = k12::KangarooTwelve::from_core(k12::KangarooTwelveCore::new(customization));
    k12_state.update(input);
    let mut k12_reader = k12_state.finalize_xof();
    let mut k12_output = vec![0; num_output_bytes];
    k12_reader.read(&mut k12_output);
    assert_eq!(output, k12_output);

    // finally, check that the tiny-keccak crate gives the same answer
    let mut tk_state = tiny_keccak::KangarooTwelve::new(customization);
    <tiny_keccak::KangarooTwelve<&[u8]> as tiny_keccak::Hasher>::update(&mut tk_state, input);
    let mut tk_xof = tk_state.into_xof();
    let mut tk_output = vec![0; num_output_bytes];
    tk_xof.squeeze(&mut tk_output);
    assert_eq!(output, tk_output);

    hex::encode(output)
}

// the KT128 ones are from https://eprint.iacr.org/2016/770.pdf,
// the KT256 ones are from
// https://datatracker.ietf.org/doc/pdf/draft-irtf-cfrg-kangarootwelve-14

#[test]
fn test_vector_01() {
    // KT128(M=empty, C=empty, 32 bytes):
    let expected = "1ac2d450fc3b4205d19da7bfca1b37513c0803577ac7167f06fe2ce1f0ef39e5";
    assert_eq!(expected, kt128_hex(&[], &[], 32));
}

#[test]
fn test_vector_02() {
    // KT128(M=empty, C=empty, 64 bytes):
    let expected = "1ac2d450fc3b4205d19da7bfca1b37513c0803577ac7167f06fe2ce1f0ef39e54269c056b8c82e48276038b6d292966cc07a3d4645272e31ff38508139eb0a71";
    assert_eq!(expected, kt128_hex(&[], &[], 64));
}

#[test]
fn test_vector_03() {
    // KT128(M=empty, C=empty, 10032 bytes), last 32 bytes:
    let expected = "e8dc563642f7228c84684c898405d3a834799158c079b12880277a1d28e2ff6d";
    let out = kt128_hex(&[], &[], 10032);
    assert_eq!(expected, &out[out.len() - 64..]);
}

#[test]
fn test_vector_04() {
    // KT128(M=pattern 0x00 to 0xFA for 17^0 bytes, C=empty, 32 bytes):
    let expected = "2bda92450e8b147f8a7cb629e784a058efca7cf7d8218e02d345dfaa65244a1f";
    let mut input = [0];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_05() {
    // KT128(M=pattern 0x00 to 0xFA for 17^1 bytes, C=empty, 32 bytes):
    let expected = "6bf75fa2239198db4772e36478f8e19b0f371205f6a9a93a273f51df37122888";
    let mut input = vec![0; 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_06() {
    // KT128(M=pattern 0x00 to 0xFA for 17^2 bytes, C=empty, 32 bytes):
    let expected = "0c315ebcdedbf61426de7dcf8fb725d1e74675d7f5327a5067f367b108ecb67c";
    let mut input = vec![0; 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_07() {
    // KT128(M=pattern 0x00 to 0xFA for 17^3 bytes, C=empty, 32 bytes):
    let expected = "cb552e2ec77d9910701d578b457ddf772c12e322e4ee7fe417f92c758f0d59d0";
    let mut input = vec![0; 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_08() {
    // KT128(M=pattern 0x00 to 0xFA for 17^4 bytes, C=empty, 32 bytes):
    let expected = "8701045e22205345ff4dda05555cbb5c3af1a771c2b89baef37db43d9998b9fe";
    let mut input = vec![0; 17 * 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_09() {
    // KT128(M=pattern 0x00 to 0xFA for 17^5 bytes, C=empty, 32 bytes):
    let expected = "844d610933b1b9963cbdeb5ae3b6b05cc7cbd67ceedf883eb678a0a8e0371682";
    let mut input = vec![0; 17 * 17 * 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_10() {
    // KT128(M=pattern 0x00 to 0xFA for 17^6 bytes, C=empty, 32 bytes):
    let expected = "3c390782a8a4e89fa6367f72feaaf13255c8d95878481d3cd8ce85f58e880af8";
    let mut input = vec![0; 17 * 17 * 17 * 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt128_hex(&input, &[], 32));
}

#[test]
fn test_vector_11() {
    // KT128(M=0 times byte 0xFF, C=pattern 0x00 to 0xFA for 41^0 bytes, 32 bytes):
    let expected = "fab658db63e94a246188bf7af69a133045f46ee984c56e3c3328caaf1aa1a583";
    let mut customization = [0];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt128_hex(&[], &customization, 32));
}

#[test]
fn test_vector_12() {
    // KT128(M=1 times byte 0xFF, C=pattern 0x00 to 0xFA for 41^1 bytes, 32 bytes):
    let expected = "d848c5068ced736f4462159b9867fd4c20b808acc3d5bc48e0b06ba0a3762ec4";
    let input = [0xff];
    let mut customization = vec![0; 41];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt128_hex(&input, &customization, 32));
}

#[test]
fn test_vector_13() {
    // KT128(M=3 times byte 0xFF, C=pattern 0x00 to 0xFA for 41^2 bytes, 32 bytes):
    let expected = "c389e5009ae57120854c2e8c64670ac01358cf4c1baf89447a724234dc7ced74";
    let input = [0xff; 3];
    let mut customization = vec![0; 41 * 41];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt128_hex(&input, &customization, 32));
}

#[test]
fn test_vector_14() {
    // KT128(M=7 times byte 0xFF, C=pattern 0x00 to 0xFA for 41^3 bytes, 32 bytes):
    let expected = "75d2f86a2e644566726b4fbcfc5657b9dbcf070c7b0dca06450ab291d7443bcf";
    let input = [0xff; 7];
    let mut customization = vec![0; 41 * 41 * 41];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt128_hex(&input, &customization, 32));
}

#[test]
fn test_vector_15() {
    // KT256(M=empty, C=empty, 64 bytes):
    let expected = "b23d2e9cea9f4904e02bec06817fc10ce38ce8e93ef4c89e6537076af8646404e3e8b68107b8833a5d30490aa33482353fd4adc7148ecb782855003aaebde4a9";
    assert_eq!(expected, kt256_hex(&[], &[], 64));
}

#[test]
fn test_vector_16() {
    // KT256(M=empty, C=empty, 128 bytes):
    let expected = "b23d2e9cea9f4904e02bec06817fc10ce38ce8e93ef4c89e6537076af8646404e3e8b68107b8833a5d30490aa33482353fd4adc7148ecb782855003aaebde4a9b0925319d8ea1e121a609821ec19efea89e6d08daee1662b69c840289f188ba860f55760b61f82114c030c97e5178449608ccd2cd2d919fc7829ff69931ac4d0";
    assert_eq!(expected, kt256_hex(&[], &[], 128));
}

#[test]
fn test_vector_17() {
    // KT256(M=empty, C=empty, 10064 bytes):
    let expected = "ad4a1d718cf950506709a4c33396139b4449041fc79a05d68da35f1e453522e056c64fe94958e7085f2964888259b9932752f3ccd855288efee5fcbb8b563069";
    let out = kt256_hex(&[], &[], 10064);
    assert_eq!(expected, &out[out.len() - 128..]);
}

#[test]
fn test_vector_18() {
    // KT256(M=pattern 0x00 to 0xfa for 17^0 bytes, C=empty, 64 bytes):
    let expected = "0d005a194085360217128cf17f91e1f71314efa5564539d444912e3437efa17f82db6f6ffe76e781eaa068bce01f2bbf81eacb983d7230f2fb02834a21b1ddd0";
    let mut input = [0];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_19() {
    // KT256(M=pattern 0x00 to 0xfa for 17^1 bytes, C=empty, 64 bytes):
    let expected = "1ba3c02b1fc514474f06c8979978a9056c8483f4a1b63d0dccefe3a28a2f323e1cdcca40ebf006ac76ef0397152346837b1277d3e7faa9c9653b19075098527b";
    let mut input = vec![0; 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_20() {
    // KT256(M=pattern 0x00 to 0xfa for 17^2 bytes, C=empty, 64 bytes):
    let expected = "de8ccbc63e0f133ebb4416814d4c66f691bbf8b6a61ec0a7700f836b086cb029d54f12ac7159472c72db118c35b4e6aa213c6562caaa9dcc518959e69b10f3ba";
    let mut input = vec![0; 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_21() {
    // KT256(M=pattern 0x00 to 0xfa for 17^3 bytes, C=empty, 64 bytes):
    let expected = "647efb49fe9d717500171b41e7f11bd491544443209997ce1c2530d15eb1ffbb598935ef954528ffc152b1e4d731ee2683680674365cd191d562bae753b84aa5";
    let mut input = vec![0; 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_22() {
    // KT256(M=pattern 0x00 to 0xfa for 17^4 bytes, C=empty, 64 bytes):
    let expected = "b06275d284cd1cf205bcbe57dccd3ec1ff6686e3ed15776383e1f2fa3c6ac8f08bf8a162829db1a44b2a43ff83dd89c3cf1ceb61ede659766d5ccf817a62ba8d";
    let mut input = vec![0; 17 * 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_23() {
    // KT256(M=pattern 0x00 to 0xfa for 17^5 bytes, C=empty, 64 bytes):
    let expected = "9473831d76a4c7bf77ace45b59f1458b1673d64bcd877a7c66b2664aa6dd149e60eab71b5c2bab858c074ded81ddce2b4022b5215935c0d4d19bf511aeeb0772";
    let mut input = vec![0; 17 * 17 * 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_24() {
    // KT256(M=pattern 0x00 to 0xfa for 17^6 bytes, C=empty, 64 bytes):
    let expected = "0652b740d78c5e1f7c8dcc1777097382768b7ff38f9a7a20f29f413bb1b3045b31a5578f568f911e09cf44746da84224a5266e96a4a535e871324e4f9c7004da";
    let mut input = vec![0; 17 * 17 * 17 * 17 * 17 * 17];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_25() {
    // KT256(M=empty, C=pattern 0x00 to 0xfa for 17^0 bytes, 64 bytes):
    let expected = "9280f5cc39b54a5a594ec63de0bb99371e4609d44bf845c2f5b8c316d72b159811f748f23e3fabbe5c3226ec96c62186df2d33e9df74c5069ceecbb4dd10eff6";
    let mut customization = [0];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt256_hex(&[], &customization, 64));
}

#[test]
fn test_vector_26() {
    // KT256(M=1 times byte 0xff, C=pattern 0x00 to 0xfa for 41 bytes, 64 bytes):
    let expected = "47ef96dd616f200937aa7847e34ec2feae8087e3761dc0f8c1a154f51dc9ccf845d7adbce57ff64b639722c6a1672e3bf5372d87e00aff89be97240756998853";
    let input = [0xff];
    let mut customization = vec![0; 41];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt256_hex(&input, &customization, 64));
}

#[test]
fn test_vector_27() {
    // KT256(M=3 times byte 0xff, C=pattern 0x00 to 0xfa for 41^2 bytes, 64 bytes):
    let expected = "3b48667a5051c5966c53c5d42b95de451e05584e7806e2fb765eda959074172cb438a9e91dde337c98e9c41bed94c4e0aef431d0b64ef2324f7932caa6f54969";
    let input = [0xff; 3];
    let mut customization = vec![0; 41 * 41];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt256_hex(&input, &customization, 64));
}

#[test]
fn test_vector_28() {
    // KT256(M=7 times byte 0xff, C=pattern 0x00 to 0xfa for 41^3 bytes, 64 bytes):
    let expected = "e0911cc00025e1540831e266d94add9b98712142b80d2629e643aac4efaf5a3a30a88cbf4ac2a91a2432743054fbcc9897670e86ba8cec2fc2ace9c966369724";
    let input = [0xff; 7];
    let mut customization = vec![0; 41 * 41 * 41];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt256_hex(&input, &customization, 64));
}

#[test]
fn test_vector_29() {
    // KT256(M=pattern 0x00 to 0xfa for 8191 bytes, C=empty, 64 bytes):
    let expected = "3081434d93a4108d8d8a3305b89682cebedc7ca4ea8a3ce869fbb73cbe4a58eef6f24de38ffc170514c70e7ab2d01f03812616e863d769afb3753193ba045b20";
    let mut input = vec![0; 8191];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_30() {
    // KT256(M=pattern 0x00 to 0xfa for 8192 bytes, C=empty, 64 bytes):
    let expected = "c6ee8e2ad3200c018ac87aaa031cdac22121b412d07dc6e0dccbb53423747e9a1c18834d99df596cf0cf4b8dfafb7bf02d139d0c9035725adc1a01b7230a41fa";
    let mut input = vec![0; 8192];
    fill_pattern(&mut input);
    assert_eq!(expected, kt256_hex(&input, &[], 64));
}

#[test]
fn test_vector_31() {
    // KT256(M=pattern 0x00 to 0xfa for 8192 bytes, C=pattern 0x00 to 0xfa for 8189 bytes, 64 bytes):
    let expected = "74e47879f10a9c5d11bd2da7e194fe57e86378bf3c3f7448eff3c576a0f18c5caae0999979512090a7f348af4260d4de3c37f1ecaf8d2c2c96c1d16c64b12496";
    let mut input = vec![0; 8192];
    fill_pattern(&mut input);
    let mut customization = vec![0; 8189];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt256_hex(&input, &customization, 64));
}

#[test]
fn test_vector_32() {
    // KT256(M=pattern 0x00 to 0xfa for 8192 bytes, C=pattern 0x00 to 0xfa for 8190 bytes, 64 bytes):
    let expected = "f4b5908b929ffe01e0f79ec2f21243d41a396b2e7303a6af1d6399cd6c7a0a2dd7c4f607e8277f9c9b1cb4ab9ddc59d4b92d1fc7558441f1832c3279a4241b8b";
    let mut input = vec![0; 8192];
    fill_pattern(&mut input);
    let mut customization = vec![0; 8190];
    fill_pattern(&mut customization);
    assert_eq!(expected, kt256_hex(&input, &customization, 64));
}
