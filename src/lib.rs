//! A Rust wrapper around the [eXtended Keccak Code Package
//! implementation](https://github.com/XKCP/K12) of the
//! [KangarooTwelve](https://keccak.team/kangarootwelve.html) cryptographic
//! hash function
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use marsupial::{KT128, Hasher};
//! // hash an input all at once
//! let hash1 = marsupial::hash::<KT128>(b"foobarbaz");
//!
//! // hash an input incrementally
//! let mut hasher = Hasher::<KT128>::new();
//! hasher.update(b"foo");
//! hasher.update(b"bar");
//! hasher.update(b"baz");
//! let hash2 = hasher.finalize();
//! assert_eq!(hash1, hash2);
//!
//! // extended output. `OutputReader` also implements `Read`
//! let mut hasher = Hasher::<KT128>::new();
//! hasher.update(b"foobarbaz");
//! let mut output_reader = hasher.finalize_xof();
//! let mut output = [0; 1000];
//! output_reader.squeeze(&mut output);
//! assert_eq!(&output[..32], hash1.as_bytes());
//!
//! // emit the hash as hexadecimal (does not work for now)
//! //println!("{}", hash1.to_hex());
//! # Ok(())
//! # }
//! ```

use std::{fmt, marker::PhantomData, mem::MaybeUninit};

#[cfg(test)]
mod test;

/// An internal trait used to prevent foreign implementations of the
/// [`SecurityLevel`] trait
trait Sealed {}

/// A trait representing a valid [`Hasher`] security level
#[allow(private_bounds)]
pub trait SecurityLevel: Sealed {
    /// The security strength level, represented in terms of bits
    const BITS: usize;

    /// The array length of the canonical [`struct@Hash`] associated with
    /// this [`SecurityLevel`]
    const HASH_ARRAY_LENGTH: usize;

    /// The canonical [`struct@Hash`] length associated with this
    /// [`SecurityLevel`]
    type Hash: Default + fmt::Debug + Eq + PartialEq + Into<Vec<u8>> + HashContainer;
}

/// The security strength level associated with the KT128 extendable output
/// function
pub struct KT128;

impl Sealed for KT128 {}

impl SecurityLevel for KT128 {
    const BITS: usize = 128;
    const HASH_ARRAY_LENGTH: usize = 32;
    type Hash = Hash<32>;
}

/// The security strength level associated with the KT256 extendable output
/// function
pub struct KT256;

impl Sealed for KT256 {}

impl SecurityLevel for KT256 {
    const BITS: usize = 256;
    const HASH_ARRAY_LENGTH: usize = 64;
    type Hash = Hash<64>;
}

/// An internal trait used to allow the [`struct@Hash`] type to be polymorphic
/// over the number of bytes it contains while still working as a return
/// type from [`Hasher`] methods
trait HashContainer {
    /// A raw pointer to the memory region containing the hash
    fn ptr(&mut self) -> *mut u8;

    /// The length of the memory region containing the hash (in bytes)
    fn len() -> usize;
}

/// Hash a slice of bytes all at once. For multiple writes, the optional
/// customization string, or extended output bytes, see [`Hasher`]
///
/// The `N` parameter indicates the security strength level in number of bits.
/// Valid values for it are:
///
/// - [`KT128`]
/// - [`KT256`]
///
/// Any other value will fail to compile
///
/// [`Hasher`]: struct.Hasher.html
pub fn hash<N>(input: &[u8]) -> N::Hash
where
    N: SecurityLevel,
{
    let mut hasher = Hasher::<N>::new();
    hasher.update(input);
    hasher.finalize()
}

/// An incremental hash state that can accept any number of writes
///
/// The `N` parameter indicates the security strength level in number of bits.
/// Valid values for it are:
///
/// - [`KT128`]
/// - [`KT256`]
///
/// Any other value will fail to compile
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use marsupial::{KT128, Hasher};
/// // hash an input incrementally
/// let mut hasher = Hasher::<KT128>::new();
/// hasher.update(b"foo");
/// hasher.update(b"bar");
/// hasher.update(b"baz");
/// assert_eq!(hasher.finalize(), marsupial::hash::<KT128>(b"foobarbaz"));
///
/// // extended output. `OutputReader` also implements `Read` and `Seek`
/// let mut hasher = Hasher::<KT128>::new();
/// hasher.update(b"foobarbaz");
/// let mut output = [0; 1000];
/// let mut output_reader = hasher.finalize_xof();
/// output_reader.squeeze(&mut output);
/// assert_eq!(&output[..32], marsupial::hash::<KT128>(b"foobarbaz").as_bytes());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Hasher<N>(marsupial_sys::KangarooTwelve_Instance, PhantomData<N>);

impl<N> Hasher<N>
where
    N: SecurityLevel,
{
    /// The number of bytes hashed or output per block
    pub const RATE: usize = (1600 - (2 * N::BITS)) / 8;

    /// Construct a new [`Hasher`] for the regular hash function
    pub fn new() -> Self {
        let mut inner = MaybeUninit::uninit();
        let inner = unsafe {
            let ret =
                marsupial_sys::KangarooTwelve_Initialize(inner.as_mut_ptr(), N::BITS as i32, 0);

            //NOTE: in practice, this does not return anything other than 0.
            //      this may, however, be changed in an update
            debug_assert_eq!(0, ret);

            inner.assume_init()
        };

        //NOTE: this is probably the only thing worth checking for
        debug_assert_eq!(inner.phase, 1);
        Self(inner, PhantomData)
    }

    /// Add input bytes to the hash state. You can call this any number of
    /// times, until the [`Hasher`] is finalized
    pub fn update(&mut self, input: &[u8]) {
        unsafe {
            let ret =
                marsupial_sys::KangarooTwelve_Update(&mut self.0, input.as_ptr(), input.len());
            debug_assert_eq!(0, ret);
        }
    }

    /// Finalize the hash state, consuming the [`Hasher`], and return the
    /// [`struct@Hash`] of the input. This method is equivalent to
    /// [`finalize_custom`](#method.finalize_custom) with an empty
    /// customization string
    pub fn finalize(self) -> N::Hash {
        self.finalize_custom(&[])
    }

    /// Finalize the hash state, consuming the [`Hasher`], and return the
    /// [`struct@Hash`] of the input
    pub fn finalize_custom(mut self, customization: &[u8]) -> N::Hash {
        let mut hash = N::Hash::default();
        unsafe {
            let ret = marsupial_sys::KangarooTwelve_Final(
                &mut self.0,
                std::ptr::null_mut(),
                customization.as_ptr(),
                customization.len(),
            );
            debug_assert_eq!(0, ret);
            let ret =
                marsupial_sys::KangarooTwelve_Squeeze(&mut self.0, hash.ptr(), N::Hash::len());
            debug_assert_eq!(0, ret);
        }
        hash
    }

    /// Finalize the hash state, consuming the [`Hasher`] and returning
    /// an [`OutputReader`], which can supply any number of output bytes.
    /// This method is equivalent to
    /// [`finalize_custom_xof`](#method.finalize_custom_xof) with an empty
    /// customization string
    ///
    /// [`OutputReader`]: struct.OutputReader.html
    pub fn finalize_xof(self) -> OutputReader {
        self.finalize_custom_xof(&[])
    }

    /// Finalize the hash state, consuming the [`Hasher`] and returning an
    /// [`OutputReader`], which can supply any number of output bytes
    ///
    /// [`OutputReader`]: struct.OutputReader.html
    pub fn finalize_custom_xof(mut self, customization: &[u8]) -> OutputReader {
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

impl<N> Default for Hasher<N>
where
    N: SecurityLevel,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N> fmt::Debug for Hasher<N>
where
    N: SecurityLevel,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hasher").finish_non_exhaustive()
    }
}

/// An output of the default size, 32 bytes, which provides constant-time
/// equality checking
///
/// `Hash` implements [`From`] and [`Into`] for `[u8; N]`, and it provides an
/// explicit [`as_bytes`] method returning `&[u8; N]`. However, byte arrays
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
/// let hash: Hash<32> = hash_array.into();
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
//NOTE: this is fine because our manual `PartialEq` implementation doesn't
//      deviate from how rust would determine equality normally
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Clone, Copy, Hash)]
pub struct Hash<const N: usize>([u8; N]);

impl<const N: usize> Hash<N> {
    /// The bytes of the [`struct@Hash`]. Note that byte arrays don't provide
    /// constant-time equality checking, so if  you need to compare hashes,
    /// prefer the [`struct@Hash`] type
    #[inline]
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> From<[u8; N]> for Hash<N> {
    #[inline]
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> From<Hash<N>> for Vec<u8> {
    #[inline]
    fn from(hash: Hash<N>) -> Self {
        hash.0.into()
    }
}

impl<const N: usize> From<Hash<N>> for [u8; N] {
    #[inline]
    fn from(hash: Hash<N>) -> Self {
        hash.0
    }
}

/// This implementation is constant-time
impl<const N: usize> PartialEq for Hash<N> {
    #[inline]
    fn eq(&self, other: &Hash<N>) -> bool {
        constant_time_eq::constant_time_eq_n(&self.0, &other.0)
    }
}

/// This implementation is constant-time
impl<const N: usize> PartialEq<[u8; N]> for Hash<N> {
    #[inline]
    fn eq(&self, other: &[u8; N]) -> bool {
        constant_time_eq::constant_time_eq_n(&self.0, other)
    }
}

impl<const N: usize> Eq for Hash<N> {}

impl<const N: usize> fmt::Debug for Hash<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Hash").finish()
    }
}

impl<const N: usize> Default for Hash<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> HashContainer for Hash<N> {
    #[inline]
    fn ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    #[inline]
    fn len() -> usize {
        N
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
