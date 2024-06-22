//! A Rust wrapper around the [eXtended Keccak Code Package
//! implementation](https://github.com/XKCP/K12) of the
//! [KangarooTwelve](https://keccak.team/kangarootwelve.html) cryptographic
//! hash function
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use marsupial::Hasher;
//! // hash an input all at once
//! let hash1 = marsupial::hash::<128>(b"foobarbaz");
//!
//! // hash an input incrementally
//! let mut hasher = Hasher::<128>::new();
//! hasher.update(b"foo");
//! hasher.update(b"bar");
//! hasher.update(b"baz");
//! let hash2 = hasher.finalize();
//! assert_eq!(hash1, hash2);
//!
//! // extended output. `OutputReader` also implements `Read`
//! let mut hasher = Hasher::<128>::new();
//! hasher.update(b"foobarbaz");
//! let mut output_reader = hasher.finalize_xof();
//! let mut output = [0; 1000];
//! output_reader.squeeze(&mut output);
//! assert_eq!(&output[..32], hash1.as_bytes());
//!
//! // emit the hash as hexadecimal
//! println!("{}", hash1.to_hex());
//! # Ok(())
//! # }
//! ```

use arrayvec::ArrayString;
use std::{fmt, mem::MaybeUninit};

#[cfg(test)]
mod test;

/// Helpers used to enforce the correctness of the `SECURITY_LEVEL` parameter
/// of the [`Hasher`]
pub mod bounds {
    trait SealedIsOk {}

    /// Trait implemented on correct [`SecurityLevel`]s
    #[allow(private_bounds)]
    pub trait IsOk: SealedIsOk {}

    impl<T> IsOk for T where T: SealedIsOk {}

    /// Container for the `SECURITY_LEVEL` parameter
    pub struct SecurityLevel<const SECURITY_LEVEL: usize>;

    impl SealedIsOk for SecurityLevel<128> {}
    impl SealedIsOk for SecurityLevel<256> {}
}

/// Hash a slice of bytes all at once. For multiple writes, the optional
/// customization string, or extended output bytes, see [`Hasher`]
///
/// The `SECURITY_LEVEL` parameter indicates the security strength level in
/// terms of bits. Valid values for it are:
///
/// - `128usize` - the `KT128` hash function
/// - `256usize` - the `KT256` hash function
///
/// Any other value will fail to compile
///
/// [`Hasher`]: struct.Hasher.html
pub fn hash<const SECURITY_LEVEL: usize>(input: &[u8]) -> Hash
where
    bounds::SecurityLevel<SECURITY_LEVEL>: bounds::IsOk,
{
    let mut hasher = Hasher::<SECURITY_LEVEL>::new();
    hasher.update(input);
    hasher.finalize()
}

/// An incremental hash state that can accept any number of writes
///
/// The `SECURITY_LEVEL` parameter indicates the security strength level in
/// terms of bits. Valid values for it are:
///
/// - `128usize` - the `KT128` hash function
/// - `256usize` - the `KT256` hash function
///
/// Any other value will fail to compile
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use marsupial::Hasher;
/// // hash an input incrementally
/// let mut hasher = Hasher::<128>::new();
/// hasher.update(b"foo");
/// hasher.update(b"bar");
/// hasher.update(b"baz");
/// assert_eq!(hasher.finalize(), marsupial::hash::<128>(b"foobarbaz"));
///
/// // extended output. `OutputReader` also implements `Read` and `Seek`
/// let mut hasher = Hasher::<128>::new();
/// hasher.update(b"foobarbaz");
/// let mut output = [0; 1000];
/// let mut output_reader = hasher.finalize_xof();
/// output_reader.squeeze(&mut output);
/// assert_eq!(&output[..32], marsupial::hash::<128>(b"foobarbaz").as_bytes());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Hasher<const SECURITY_LEVEL: usize>(marsupial_sys::KangarooTwelve_Instance);

impl<const SECURITY_LEVEL: usize> Hasher<SECURITY_LEVEL>
where
    bounds::SecurityLevel<SECURITY_LEVEL>: bounds::IsOk,
{
    /// The number of bytes hashed or output per block
    pub const RATE: usize = (1600 - (2 * SECURITY_LEVEL)) / 8;

    /// Construct a new [`Hasher`] for the regular hash function
    pub fn new() -> Self {
        let mut inner = MaybeUninit::uninit();
        let inner = unsafe {
            let ret = marsupial_sys::KangarooTwelve_Initialize(
                inner.as_mut_ptr(),
                SECURITY_LEVEL as i32,
                0,
            );
            debug_assert_eq!(0, ret);
            inner.assume_init()
        };
        // These asserts help check that our struct definitions agree with C
        debug_assert_eq!(0, inner.fixedOutputLength);
        debug_assert_eq!(0, inner.blockNumber);
        debug_assert_eq!(0, inner.queueAbsorbedLen);
        debug_assert_eq!(inner.phase, 1);
        debug_assert_eq!(0, inner.finalNode.byteIOIndex);
        debug_assert_eq!(0, inner.finalNode.squeezing);
        Self(inner)
    }

    /// Add input bytes to the hash state. You can call this any number of
    /// times, until the [`Hasher`] is finalized
    pub fn update(&mut self, input: &[u8]) {
        assert_eq!(self.0.phase, 1, "this instance has already been finalized");
        unsafe {
            let ret =
                marsupial_sys::KangarooTwelve_Update(&mut self.0, input.as_ptr(), input.len());
            debug_assert_eq!(0, ret);
        }
    }

    /// Finalize the hash state and return the [`struct@Hash`] of the input. This
    /// method is equivalent to [`finalize_custom`](#method.finalize_custom)
    /// with an empty customization string
    ///
    /// You can only finalize a [`Hasher`] once. Additional calls to any of
    /// the finalize methods will panic
    pub fn finalize(&mut self) -> Hash {
        self.finalize_custom(&[])
    }

    /// Finalize the hash state using the given customization string and
    /// return the [`struct@Hash`] of the input
    ///
    /// You can only finalize a [`Hasher`] once. Additional calls to any of
    /// the finalize methods will panic
    pub fn finalize_custom(&mut self, customization: &[u8]) -> Hash {
        assert_eq!(self.0.phase, 1, "this instance has already been finalized");
        let mut bytes = [0; 32];
        unsafe {
            let ret = marsupial_sys::KangarooTwelve_Final(
                &mut self.0,
                std::ptr::null_mut(),
                customization.as_ptr(),
                customization.len(),
            );
            debug_assert_eq!(0, ret);
            let ret =
                marsupial_sys::KangarooTwelve_Squeeze(&mut self.0, bytes.as_mut_ptr(), bytes.len());
            debug_assert_eq!(0, ret);
        }
        bytes.into()
    }

    /// Finalize the hash state and return an [`OutputReader`], which can
    /// supply any number of output bytes. This method is equivalent to
    /// [`finalize_custom_xof`](#method.finalize_custom_xof) with an empty
    /// customization string
    ///
    /// You can only finalize a [`Hasher`] once. Additional calls to any of
    /// the finalize methods will panic
    ///
    /// [`OutputReader`]: struct.OutputReader.html
    pub fn finalize_xof(&mut self) -> OutputReader {
        self.finalize_custom_xof(&[])
    }

    /// Finalize the hash state and return an [`OutputReader`], which can
    /// supply any number of output bytes
    ///
    /// You can only finalize a [`Hasher`] once. Additional calls to any of
    /// the finalize methods will panic
    ///
    /// [`OutputReader`]: struct.OutputReader.html
    pub fn finalize_custom_xof(&mut self, customization: &[u8]) -> OutputReader {
        assert_eq!(self.0.phase, 1, "this instance has already been finalized");
        unsafe {
            let ret = marsupial_sys::KangarooTwelve_Final(
                &mut self.0,
                std::ptr::null_mut(),
                customization.as_ptr(),
                customization.len(),
            );
            debug_assert_eq!(0, ret);
        }
        OutputReader(self.0)
    }
}

impl<const SECURITY_LEVEL: usize> Default for Hasher<SECURITY_LEVEL>
where
    bounds::SecurityLevel<SECURITY_LEVEL>: bounds::IsOk,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const SECURITY_LEVEL: usize> fmt::Debug for Hasher<SECURITY_LEVEL> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hasher").finish_non_exhaustive()
    }
}

/// An output of the default size, 32 bytes, which provides constant-time
/// equality checking
///
/// `Hash` implements [`From`] and [`Into`] for `[u8; 32]`, and it provides an
/// explicit [`as_bytes`] method returning `&[u8; 32]`. However, byte arrays
/// and slices don't provide constant-time equality checking, which is often a
/// security requirement in software that handles private data. `Hash` doesn't
/// implement [`Deref`] or [`AsRef`], to avoid situations where a type
/// conversion happens implicitly and the constant-time property is
/// accidentally lost
///
/// `Hash` provides the [`to_hex`] method for converting to hexadecimal. It
/// doesn't directly support converting from hexadecimal, but here's an
/// example of doing that with the [`hex`] crate:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use marsupial::Hash;
/// # use std::convert::TryInto;
/// let hash_hex = "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24";
/// let hash_bytes = hex::decode(hash_hex)?;
/// let hash_array: [u8; 32] = hash_bytes[..].try_into()?;
/// let hash: Hash = hash_array.into();
/// # Ok(())
/// # }
/// ```
///
/// [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
/// [`Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
/// [`as_bytes`]: #method.as_bytes
/// [`Deref`]: https://doc.rust-lang.org/stable/std/ops/trait.Deref.html
/// [`AsRef`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
/// [`to_hex`]: #method.to_hex
/// [`hex`]: https://crates.io/crates/hex
#[derive(Clone, Copy, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    /// The bytes of the [`struct@Hash`]. Note that byte arrays don't provide
    /// constant-time equality checking, so if  you need to compare hashes,
    /// prefer the [`struct@Hash`] type
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The hexadecimal encoding of the [`struct@Hash`]. The returned [`ArrayString`]
    /// is of a fixed size and doesn't allocate memory on the heap. Note that
    /// [`ArrayString`] doesn't provide constant-time equality checking, so if
    /// you need to compare hashes, prefer the `Hash` type.
    ///
    /// [`ArrayString`]: https://docs.rs/arrayvec/0.5.1/arrayvec/struct.ArrayString.html
    pub fn to_hex(&self) -> ArrayString<{ 2 * 32 }> {
        let mut s = ArrayString::new();
        let table = b"0123456789abcdef";
        for &b in self.0.iter() {
            s.push(table[(b >> 4) as usize] as char);
            s.push(table[(b & 0xf) as usize] as char);
        }
        s
    }
}

impl From<[u8; 32]> for Hash {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Hash> for [u8; 32] {
    #[inline]
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

/// This implementation is constant-time
impl PartialEq for Hash {
    #[inline]
    fn eq(&self, other: &Hash) -> bool {
        constant_time_eq::constant_time_eq_32(&self.0, &other.0)
    }
}

/// This implementation is constant-time
impl PartialEq<[u8; 32]> for Hash {
    #[inline]
    fn eq(&self, other: &[u8; 32]) -> bool {
        constant_time_eq::constant_time_eq_32(&self.0, other)
    }
}

impl Eq for Hash {}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Hash").field(&self.to_hex()).finish()
    }
}

/// An incremental reader for extended output, returned by
/// [`Hasher::finalize_xof`](struct.Hasher.html#method.finalize_xof) and
/// [`Hasher::finalize_custom_xof`](struct.Hasher.html#method.finalize_custom_xof)
#[derive(Clone)]
pub struct OutputReader(marsupial_sys::KangarooTwelve_Instance);

impl OutputReader {
    /// Fill a buffer with output bytes and advance the position of the
    /// [`OutputReader`]
    ///
    /// This is equivalent to [`Read::read`], except that it
    /// doesn't return a `Result`. Both methods always fill the entire buffer
    ///
    /// [`Read::read`]: #method.read
    pub fn squeeze(&mut self, buf: &mut [u8]) {
        debug_assert_eq!(self.0.phase, 3, "this instance has not yet been finalized");
        unsafe {
            let ret =
                marsupial_sys::KangarooTwelve_Squeeze(&mut self.0, buf.as_mut_ptr(), buf.len());
            debug_assert_eq!(0, ret);
        }
    }
}

// Don't derive(Debug), because the state may be secret
impl fmt::Debug for OutputReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OutputReader").finish_non_exhaustive()
    }
}

impl std::io::Read for OutputReader {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.squeeze(buf);
        Ok(buf.len())
    }
}
