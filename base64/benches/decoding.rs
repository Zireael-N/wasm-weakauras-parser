use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use weakauras_parser_base64 as wa_base64;

pub fn decoding_benchmark(c: &mut Criterion) {
    const KB: usize = 1024;

    let mut group = c.benchmark_group("decode");
    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB, 1024 * KB].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        let data: Vec<_> = (b'0'..=b'9')
            .chain(b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .chain(b'('..=b')')
            .cycle()
            .take(*size)
            .collect();

        let capacity = data.len() * 3 / 4;

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter_batched_ref(
                || Vec::with_capacity(capacity),
                |buffer| unsafe { wa_base64::decode_scalar(&data, buffer) },
                BatchSize::SmallInput,
            );
        });

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "ssse3"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("SSE", size), size, |b, _| {
                b.iter_batched_ref(
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe { wa_base64::decode_sse(&data, buffer) },
                    BatchSize::SmallInput,
                );
            });
        }

        #[cfg(all(
            feature = "avx2",
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "avx2"
        ))]
        {
            group.bench_with_input(BenchmarkId::new("AVX2", size), size, |b, _| {
                b.iter_batched_ref(
                    || Vec::with_capacity(capacity),
                    |buffer| unsafe { wa_base64::decode_avx2(&data, buffer) },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, decoding_benchmark);
criterion_main!(benches);
