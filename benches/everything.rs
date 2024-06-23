use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use marsupial::{KT128, KT256};
use rand::prelude::*;

const KIB: usize = 1024;

// This struct randomizes two things:
// 1. The actual bytes of input.
// 2. The page offset the input starts at.
pub struct RandomInput {
    buf: Vec<u8>,
    len: usize,
    offsets: Vec<usize>,
    offset_index: usize,
}

impl RandomInput {
    pub fn new(len: usize) -> Self {
        let page_size: usize = page_size::get();
        let mut buf = vec![0u8; len + page_size];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut buf);
        let mut offsets: Vec<usize> = (0..page_size).collect();
        offsets.shuffle(&mut rng);
        Self {
            buf,
            len,
            offsets,
            offset_index: 0,
        }
    }

    pub fn get(&mut self) -> &[u8] {
        let offset = self.offsets[self.offset_index];
        self.offset_index += 1;
        if self.offset_index >= self.offsets.len() {
            self.offset_index = 0;
        }
        &self.buf[offset..][..self.len]
    }
}

fn bench_kt128(c: &mut Criterion) {
    let mut g = c.benchmark_group("KT128");

    for n in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024].iter() {
        let bytes = n * KIB;
        g.throughput(Throughput::Bytes(bytes as u64));

        let mut marsupial_input = black_box(RandomInput::new(bytes));
        g.bench_function(BenchmarkId::new("marsupial", n), |b| {
            b.iter(|| marsupial::hash::<KT128>(marsupial_input.get()))
        });

        let mut k12_input = black_box(RandomInput::new(bytes));
        g.bench_function(BenchmarkId::new("k12", n), |b| {
            b.iter(|| {
                use digest::{ExtendableOutput, Update, XofReader};
                use k12::{KangarooTwelve, KangarooTwelveCore};

                let mut state = KangarooTwelve::from_core(KangarooTwelveCore::default());
                state.update(k12_input.get());

                let mut reader = state.finalize_xof();
                let mut output = [0; 32];
                reader.read(&mut output);
                output
            })
        });

        let mut tk_input = black_box(RandomInput::new(bytes));
        g.bench_function(BenchmarkId::new("tiny-keccak", n), |b| {
            b.iter(|| {
                use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};

                let mut state = KangarooTwelve::new(b"");
                state.update(tk_input.get());

                let mut xof = state.into_xof();
                let mut output = [0; 32];
                xof.squeeze(&mut output);
                output
            })
        });
    }
}

fn bench_kt256(c: &mut Criterion) {
    let mut g = c.benchmark_group("KT256");

    for n in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024].iter() {
        let bytes = n * KIB;
        g.throughput(Throughput::Bytes(bytes as u64));

        let mut marsupial_input = black_box(RandomInput::new(bytes));
        g.bench_function(BenchmarkId::new("marsupial", n), |b| {
            b.iter(|| marsupial::hash::<KT256>(marsupial_input.get()))
        });
    }
}

criterion_group!(benches, bench_kt128, bench_kt256);
criterion_main!(benches);
