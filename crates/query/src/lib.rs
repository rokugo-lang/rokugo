//! Query scheduler and async runtime.

pub mod arena;
mod just_about_anything;
mod name;

#[cfg(debug_assertions)]
use std::any::type_name;
use std::{
    fmt::Debug,
    future::Future,
    hash::{BuildHasherDefault, Hash},
    pin::Pin,
    sync::OnceLock,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use dashmap::{DashMap, DashSet};
use just_about_anything::JustAboutAnything;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::FxHasher;

use crate::arena::{Arena, OwnPinned};

pub use name::Name;

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
    caches_by_type:
        DashMap<Name, &'a (dyn JustAboutAnything<'a> + Sync), BuildHasherDefault<FxHasher>>,
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
    /// Note that this adds the computation to the queue immediately. This is in contrast to most
    /// futures, which will not cause any work to be performed until they're `.await`ed.
    /// However, not awaiting the future is useless, as queries are assumed to be computations
    /// without side effects - therefore you should always ensure the resulting future is awaited.
    ///
    /// # Querying efficiently
    ///
    /// Writing efficient queries generally boils down to starting as many queries as possible at a
    /// given time, and awaiting their results *after* these queries have been made. Like so:
    ///
    /// ```
    /// # use rokugo_query::*;
    /// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// # struct SomeQuery;
    /// # impl Query for SomeQuery {
    /// #     const NAME: Name = Name::new("SomeQuery");
    /// #     type Result = i32;
    /// #     async fn run(self, scheduler: &Scheduler<'_>) -> Self::Result { 123 }
    /// # }
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// struct AddTwo;
    ///
    /// impl Query for AddTwo {
    ///     const NAME: Name = Name::new("AddTwo");
    ///
    ///     type Result = i32;
    ///
    ///     async fn run(self, scheduler: &Scheduler<'_>) -> Self::Result {
    ///         // NOTE: Two queries are fired in parallel here.
    ///         let a = scheduler.query(SomeQuery);
    ///         let b = scheduler.query(SomeQuery);
    ///         a.await + b.await
    ///     }
    /// }
    /// ```
    pub fn query<Q>(&self, query: Q) -> Ongoing<Q::Result>
    where
        Q: Query,
    {
        let cache = self.cache::<Q>();

        let cell = cache.cell(self.arena, query.clone());
        if cell.get().is_none() && cache.enqueued.insert(query.clone()) {
            self.erased_queue.lock().push(Box::new(Some(query)));
        }

        Ongoing { cell }
    }
}

/// An ongoing computation of a value of type `C`.
///
/// Note that this future is fine to drop, because all computations are enqueued immediately
/// into the [`Computer`].
#[must_use]
pub struct Ongoing<'a, Q> {
    cell: &'a OnceLock<Q>,
}

impl<'a, Q> Future for Ongoing<'a, Q> {
    type Output = &'a Q;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.cell.get() {
            Poll::Ready(value)
        } else {
            Poll::Pending
        }
    }
}

/// Multithreading mode for the trampoline's polling loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PollLoop {
    /// Single-threaded mode. Works best when tasks are very short.
    #[default]
    SingleThreaded,
    /// Parallel mode. Works best when tasks are long-lived.
    Parallel,
}

/// Settings for [`Scheduler::trampoline`].
#[derive(Debug, Clone, Default)]
pub struct Trampoline {
    /// Which polling loop to use.
    pub poll_loop: PollLoop,
}

/// # Scheduling functions
///
/// Note that these require a lifetime of self which is `'a`. This is to allow for futures in the
/// scheduler to reference the scheduler itself, but this comes at the cost of requiring the
/// scheduler to be allocated in the [`Arena`] you pass to it.
impl<'a> Scheduler<'a> {
    /// Bounce in and out of scheduled tasks until all computations are done.
    pub fn trampoline(&'a self, trampoline: &Trampoline) {
        match trampoline.poll_loop {
            PollLoop::SingleThreaded => self.trampoline_single_threaded(),
            PollLoop::Parallel => self.trampoline_parallel(),
        }
    }

    pub fn request_and_trampoline<Q>(&'a self, query: Q, trampoline: &Trampoline) -> &'a Q::Result
    where
        Q: Query,
    {
        // Dropping the future here because querying queue up a tasks, which we later trampoline
        // back into a useful value.
        drop(self.query(query.clone()));
        self.trampoline(trampoline);
        self.cache()
            .cell(self.arena, query)
            .get()
            .expect("query should have computed a result into the cache")
    }

    fn trampoline_single_threaded(&'a self) {
        let mut future_queue: Vec<OwnPinned<dyn Future<Output = ()> + Send>> = vec![];
        loop {
            while let Some(mut erased_computation) = self.erased_queue.lock().pop() {
                let future = erased_computation.erased_query(self);
                future_queue.push(future);
            }

            let mut i = 0;
            while i < future_queue.len() {
                let mut pinned = self.arena.get_mut_pinned(&mut future_queue[i]);
                let poll = pinned
                    .as_mut()
                    .poll(&mut Context::from_waker(&noop_waker()));
                match poll {
                    Poll::Pending => (),
                    Poll::Ready(()) => {
                        future_queue.swap_remove(i);
                        continue;
                    }
                }
                i += 1;
            }

            if future_queue.is_empty() {
                break;
            }
        }
    }

    fn trampoline_parallel(&'a self) {
        let mut future_queue: Vec<Option<OwnPinned<dyn Future<Output = ()> + Send>>> = vec![];
        loop {
            while let Some(mut erased_computation) = self.erased_queue.lock().pop() {
                let future = erased_computation.erased_query(self);
                future_queue.push(Some(future));
            }

            future_queue.par_iter_mut().for_each(|future| {
                let mut pinned = self.arena.get_mut_pinned(
                    future
                        .as_mut()
                        .expect("future queue must be cleared of None"),
                );
                let poll = pinned
                    .as_mut()
                    .poll(&mut Context::from_waker(&noop_waker()));
                match poll {
                    Poll::Pending => (),
                    Poll::Ready(()) => {
                        *future = None;
                    }
                }
            });

            let mut i = 0;
            while i < future_queue.len() {
                if future_queue[i].is_none() {
                    future_queue.swap_remove(i);
                } else {
                    i += 1;
                }
            }

            if future_queue.is_empty() {
                break;
            }
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

    fn run<'a>(
        self,
        scheduler: &'a Scheduler<'_>,
    ) -> impl Future<Output = Self::Result> + Send + Sync + 'a;
}

// Object-safe version of `Compute`.
trait ErasedQuery: Send {
    fn erased_query<'a>(
        &mut self,
        scheduler: &'a Scheduler<'a>,
    ) -> OwnPinned<dyn Future<Output = ()> + Send + 'a>;
}

impl<Q> ErasedQuery for Option<Q>
where
    Q: Query,
{
    fn erased_query<'a>(
        &mut self,
        scheduler: &'a Scheduler<'a>,
    ) -> OwnPinned<dyn Future<Output = ()> + Send + 'a> {
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
            .as_dyn_send_future()
    }
}
