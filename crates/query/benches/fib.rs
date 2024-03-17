use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dashmap::DashMap;
use rokugo_query::{arena::Arena, Name, PollLoop, Query, Scheduler, Trampoline};

fn fib_naive(n: u32) -> u32 {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib_naive(n - 1) + fib_naive(n - 2)
    }
}

fn fib_cached(cache: &mut DashMap<u32, u32>, n: u32) -> u32 {
    if let Some(result) = cache.get(&n) {
        *result
    } else {
        let result = fib_naive(n);
        cache.insert(n, result);
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Fib(u32);

impl Query for Fib {
    const NAME: Name = Name::new("Fib");

    type Result = u32;

    async fn run(self, scheduler: &Scheduler<'_>) -> Self::Result {
        let Fib(n) = self;
        if n == 0 {
            0
        } else if n == 1 {
            1
        } else {
            let l = scheduler.query(Fib(n - 1));
            let r = scheduler.query(Fib(n - 2));
            *l.await + *r.await
        }
    }
}

fn fib_queried(n: u32) -> u32 {
    let arena = Arena::new();
    let scheduler = arena.alloc(Scheduler::new(&arena));
    *scheduler.request_and_trampoline(
        Fib(n),
        &Trampoline {
            poll_loop: PollLoop::SingleThreaded,
        },
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("fib");
    for i in 10..=10 {
        // NOTE: naive is not benchmarked because it doesn't give us any meaningful insights.
        // group.bench_with_input(BenchmarkId::new("naive", i), &i, |b, &i| {
        //     b.iter(|| fib_naive(black_box(i)));
        // });
        group.bench_with_input(BenchmarkId::new("cached", i), &i, |b, &i| {
            let mut map = DashMap::new();
            b.iter(|| fib_cached(&mut map, black_box(i)));
        });
        group.bench_with_input(BenchmarkId::new("queried", i), &i, |b, &i| {
            b.iter(|| fib_queried(black_box(i)));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
