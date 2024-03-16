//! Query scheduler and async runtime.

pub mod arena;
mod just_about_anything;
pub mod name;

use std::{
    any::type_name,
    fmt::Debug,
    future::Future,
    hash::{BuildHasherDefault, Hash},
    pin::Pin,
    sync::OnceLock,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use arena::{Arena, OwnPinned};
use dashmap::{DashMap, DashSet};
use just_about_anything::JustAboutAnything;
use name::Name;
use parking_lot::Mutex;
use rustc_hash::FxHasher;

struct Cache<'a, Q>
where
    Q: Query,
{
    cells: DashMap<Q, &'a OnceLock<Q::Result>, BuildHasherDefault<FxHasher>>,
    enqueued: DashSet<Q, BuildHasherDefault<FxHasher>>,
}

impl<'a, Q> Cache<'a, Q>
where
    Q: Query,
{
    fn new() -> Self {
        Self {
            cells: DashMap::default(),
            enqueued: DashSet::default(),
        }
    }

    fn cell(&self, arena: &'a Arena, computation: Q) -> &'a OnceLock<Q::Result> {
        *self
            .cells
            .entry(computation.clone())
            .or_insert_with(|| arena.alloc(OnceLock::default()))
    }
}

/// Queues up tasks that are to be done by the compilation loop.
pub struct Scheduler<'a> {
    /// The scheduler's arena. It is used for allocating things for the scheduler's own use, as well
    /// as letting queries allocate their own data (such as results).
    pub arena: &'a Arena,
    caches_by_type: DashMap<Name, &'a dyn JustAboutAnything<'a>, BuildHasherDefault<FxHasher>>,
    erased_queue: Mutex<Vec<Box<dyn ErasedQuery>>>,

    #[cfg(debug_assertions)]
    compute_type_names: DashMap<Name, &'static str, BuildHasherDefault<FxHasher>>,
}

impl<'a> Scheduler<'a> {
    /// Create a new scheduler.
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            arena,
            caches_by_type: DashMap::default(),
            erased_queue: Mutex::new(vec![]),

            #[cfg(debug_assertions)]
            compute_type_names: DashMap::default(),
        }
    }

    fn cache<Q>(&self) -> &Cache<'a, Q>
    where
        Q: Query,
    {
        #[cfg(debug_assertions)]
        {
            if let Some(previous_type_name) =
                self.compute_type_names.insert(Q::NAME, type_name::<Q>())
            {
                assert_eq!(
                    type_name::<Q>(),
                    previous_type_name,
                    "hash collision occurred; try using a different NAME for one of these types"
                );
            }
        }

        let cache = *self
            .caches_by_type
            .entry(Q::NAME)
            .or_insert_with(|| self.arena.alloc(Cache::<Q>::new()));

        // SAFETY: The above `let` is the only point in the code at which caches are constructed,
        // and the cache is always of type Cache<Q>.
        unsafe { just_about_anything::transmute(cache) }
    }

    /// Request a computation from the [`Computer`] that consumes tasks from this scheduler.
    ///
    /// Note that this adds the computation to the queue immediately. Therefore it is okay to _not_
    /// await the future returned by this.
    pub fn query<Q>(&self, query: Q) -> Computation<Q::Result>
    where
        Q: Query,
    {
        let cache = self.cache::<Q>();

        let cell = cache.cell(self.arena, query.clone());
        if cell.get().is_none() && cache.enqueued.insert(query.clone()) {
            self.erased_queue.lock().push(Box::new(Some(query)));
        }

        Computation { cell }
    }
}

/// An ongoing computation of a value of type `C`.
///
/// Note that this future is fine to drop, because all computations are enqueued immediately
/// into the [`Computer`].
pub struct Computation<'a, Q> {
    cell: &'a OnceLock<Q>,
}

impl<'a, Q> Future for Computation<'a, Q> {
    type Output = &'a Q;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.cell.get() {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

/// Computation loop that makes tasks make forward progress until they're all done.
pub struct Computer<'a, 's> {
    scheduler: &'s Scheduler<'a>,
    future_queue: Vec<OwnPinned<dyn Future<Output = ()> + 's>>,
}

impl<'a, 's> Computer<'a, 's> {
    /// Creates a new computer consuming from the given scheduler.
    pub fn new(scheduler: &'s Scheduler<'a>) -> Self {
        Self {
            scheduler,
            future_queue: vec![],
        }
    }
}

// Stable copy of Waker::noop.
fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Cloning just returns a new no-op raw waker
        |_| RAW,
        // `wake` does nothing
        |_| {},
        // `wake_by_ref` does nothing
        |_| {},
        // Dropping does nothing as we don't allocate anything
        |_| {},
    );
    const RAW: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);

    // SAFETY: All methods are noops.
    unsafe { Waker::from_raw(RAW) }
}

impl<'a, 's: 'a> Computer<'a, 's> {
    /// Bounce in and out of tasks until all computations are done.
    pub fn trampoline(&mut self) {
        loop {
            while let Some(mut erased_computation) = self.scheduler.erased_queue.lock().pop() {
                let future = erased_computation.erased_query(self.scheduler);
                self.future_queue.push(future);
            }
            self.future_queue.retain_mut(|future| {
                let mut pinned = self.scheduler.arena.get_mut_pinned(future);
                match pinned
                    .as_mut()
                    .poll(&mut Context::from_waker(&noop_waker()))
                {
                    Poll::Ready(()) => false,
                    Poll::Pending => true,
                }
            });
            if self.future_queue.is_empty() {
                break;
            }
        }
    }

    /// Make a request for the given computation and [`trampoline`][Self::trampoline] tasks until
    /// they're all done and have fulfilled the request. Returns the result of the request.
    pub fn request_and_trampoline<Q>(&mut self, computation: Q) -> &'a Q::Result
    where
        Q: Query,
    {
        // Dropping the future here because requesting a computation queues tasks, which we
        // later trampoline back into a useful value.
        self.scheduler.query(computation.clone());
        self.trampoline();
        self.scheduler
            .cache()
            .cell(self.scheduler.arena, computation)
            .get()
            .expect("query should have computed a result into the cache")
    }
}

/// Represents a computation type.
///
/// This can be thought of as a *function call descriptor.* It stores the arguments needed to call
/// the function. It is also equatable and hashable so that results can be memoized using a hash map.
///
/// This should generally made out of plain old data and be cheap to clone.
pub trait Query: 'static + Clone + Debug + Eq + Hash + Send + Sync {
    /// Unique name to distinguish the query from others.
    /// If the name is not unique within the current module, an assertion will be triggered.
    const NAME: Name;

    type Result: Send + Sync;

    fn run<'a>(self, scheduler: &'a Scheduler<'_>) -> impl Future<Output = Self::Result> + 'a;
}

// Object-safe version of `Compute`.
trait ErasedQuery {
    fn erased_query<'a, 's: 'a>(
        &mut self,
        scheduler: &'s Scheduler<'a>,
    ) -> OwnPinned<dyn Future<Output = ()> + 'a>;
}

impl<Q> ErasedQuery for Option<Q>
where
    Q: Query,
{
    fn erased_query<'a, 's: 'a>(
        &mut self,
        scheduler: &'s Scheduler<'a>,
    ) -> OwnPinned<dyn Future<Output = ()> + 'a> {
        let query = self.take().expect("erased_query must only be called once");
        let cache = scheduler.cache();
        let cell = cache.cell(scheduler.arena, query.clone());
        scheduler
            .arena
            .alloc_own_pinned(async move {
                let cache = scheduler.cache::<Q>();
                let future = query.clone().run(scheduler).await;
                cell.set(future)
                    .map_err(|_| ())
                    .expect("cell may only be computed once");
                cache.enqueued.remove(&query);
            })
            .as_dyn_future()
    }
}
