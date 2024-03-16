use rokugo_query::{arena::Arena, name::Name, PollLoop, Query, Scheduler, Trampoline};

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
            l.await + r.await
        }
    }
}

#[test]
fn rec_fib() {
    let arena = Arena::new();
    let scheduler = arena.alloc(Scheduler::new(&arena));
    let fib = scheduler.request_and_trampoline(Fib(30), &Trampoline::default());
    assert_eq!(*fib, 832040);
}

#[test]
fn threading() {
    let arena = Arena::new();
    let scheduler = arena.alloc(Scheduler::new(&arena));
    let fib_st = scheduler.request_and_trampoline(
        Fib(30),
        &Trampoline {
            poll_loop: PollLoop::SingleThreaded,
        },
    );
    let fib_mt = scheduler.request_and_trampoline(
        Fib(30),
        &Trampoline {
            poll_loop: PollLoop::Parallel,
        },
    );
    assert_eq!(fib_st, fib_mt);
}
