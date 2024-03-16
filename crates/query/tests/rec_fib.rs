use rokugo_query::{arena::Arena, name::Name, Computer, Query, Scheduler};

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
    let mut computer = Computer::new(scheduler);
    let fib = computer.request_and_trampoline(Fib(30));
    assert_eq!(*fib, 832040);
}
