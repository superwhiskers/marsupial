use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use digest::{ExtendableOutput, Update, XofReader};
use k12::{KangarooTwelve, KangarooTwelveCore};
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

fn bench_atonce(c: &mut Criterion, n: usize) {
    let mut g = c.benchmark_group(format!("{} kib", n));
    let bytes = n * KIB;

    g.throughput(Throughput::Bytes(bytes as u64));

    let mut marsupial_input = black_box(RandomInput::new(bytes));
    g.bench_function("marsupial", |b| {
        b.iter(|| marsupial::hash(marsupial_input.get()))
    });

    let mut k12_input = black_box(RandomInput::new(bytes));
    g.bench_function("k12", |b| {
        b.iter(|| {
            let mut state = KangarooTwelve::from_core(KangarooTwelveCore::default());
            state.update(k12_input.get());

            let mut reader = state.finalize_xof();
            let mut output = [0; 32];
            reader.read(&mut output);
            output
        })
    });
}

fn benchmark(c: &mut Criterion) {
    bench_atonce(c, 1);
    bench_atonce(c, 2);
    bench_atonce(c, 4);
    bench_atonce(c, 8);
    bench_atonce(c, 16);
    bench_atonce(c, 32);
    bench_atonce(c, 64);
    bench_atonce(c, 128);
    bench_atonce(c, 256);
    bench_atonce(c, 512);
    bench_atonce(c, 1024);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
