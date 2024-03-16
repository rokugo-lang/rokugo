use std::{
    alloc::Layout,
    fmt,
    future::Future,
    pin::Pin,
    ptr::{addr_of_mut, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use parking_lot::Mutex;

/// An allocation arena.
/// Allows for extending the lifetime of allocations to the entire compilation.
///
/// Allocation arenas additionally have a lifetime parameter, which allows an arena to store
/// references that outlives it.
pub struct Arena {
    // TODO: This should probably use a bump allocator rather than a vector of boxes.
    // Allocations that require `Drop` could be stored in a linked list to avoid the overhead
    // of reallocating the vector if it grows too big.
    // Lots of optimization opportunities here.
    allocs: Mutex<Vec<NonNull<Allocation<()>>>>,
    droppers: Mutex<Vec<Dropper>>,
    index: usize,
}

#[repr(C)]
struct Allocation<T> {
    layout: Layout,
    data: T,
}

struct Dropper {
    alloc_index: usize,
    drop_fn: unsafe fn(NonNull<Allocation<()>>),
}

static ARENA_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Arena {
    /// Creates a new arena.
    pub fn new() -> Self {
        Self {
            allocs: Mutex::new(vec![]),
            droppers: Mutex::new(vec![]),
            index: ARENA_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

    unsafe fn dropper<T>(ptr: NonNull<Allocation<()>>) {
        let ptr = ptr.cast::<Allocation<T>>();
        std::ptr::drop_in_place(addr_of_mut!((*ptr.as_ptr()).data))
    }

    fn alloc_ptr<T>(&self, value: T) -> NonNull<T> {
        let layout = Layout::new::<Allocation<T>>();
        if layout.size() == 0 {
            return NonNull::dangling();
        }

        // SAFETY: T's layout has a non-zero size because of the above check.
        let mut ptr = unsafe {
            NonNull::new(std::alloc::alloc(layout))
                .expect("allocation failed")
                .cast::<Allocation<T>>()
        };
        // SAFETY: The allocated pointer is definitely valid, because we panic on
        // allocation failure.
        unsafe {
            std::ptr::write(
                ptr.as_ptr(),
                Allocation {
                    layout,
                    data: value,
                },
            )
        };

        let erased = ptr.cast::<Allocation<()>>();
        let alloc_index = {
            let mut allocs = self.allocs.lock();
            let i = allocs.len();
            allocs.push(erased);
            i
        };
        if std::mem::needs_drop::<T>() {
            self.droppers.lock().push(Dropper {
                alloc_index,
                drop_fn: Self::dropper::<T>,
            });
        }

        // SAFETY: `ptr` is a valid pointer, therefore offsetting it and constructing a NonNull from
        // it is fine.
        unsafe { NonNull::new_unchecked(addr_of_mut!(ptr.as_mut().data)) }
    }

    /// Allocate a value in the arena and return a mutable reference to it.
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let mut ptr = self.alloc_ptr(value);
        // SAFETY: A new allocation is created every time and is not mutated until the Arena
        // needs to be dropped.
        unsafe { ptr.as_mut() }
    }

    /// Same as [`alloc`][Self::alloc], but returns a pinned reference.
    pub fn alloc_pinned<T>(&self, value: T) -> Pin<&mut T> {
        // SAFETY: The memory allocated by `alloc` lives on the heap and lives as long as &self.
        unsafe { Pin::new_unchecked(self.alloc(value)) }
    }

    /// Same as [`alloc`][Self::alloc], but returns an [`Own<T>`].
    pub fn alloc_own<T>(&self, value: T) -> Own<T> {
        let ptr = self.alloc_ptr(value);
        Own {
            ptr,
            arena_index: self.index,
        }
    }

    /// Same as [`alloc`][Self::alloc], but returns an [`OwnPinned<T>`].
    pub fn alloc_own_pinned<T>(&self, value: T) -> OwnPinned<T> {
        let ptr = self.alloc_ptr(value);
        OwnPinned {
            ptr,
            arena_index: self.index,
        }
    }

    /// Resolves a [`Ref<T>`] into a reference, if the [`Ref<T>`] was created in this arena.
    /// Otherwise returns [`DifferentArenaError`].
    pub fn try_get<T: ?Sized>(&self, re: Ref<T>) -> Result<&T, DifferentArenaError> {
        if re.arena_index == self.index {
            // SAFETY: The `if` statement checks that the pointer inside `re` belongs to this arena,
            // and since this arena is live, all pointers coming from it are live too.
            Ok(unsafe { re.ptr.as_ref() })
        } else {
            Err(DifferentArenaError)
        }
    }

    /// Same as [`try_get`][Self::try_get], but panics on error.
    pub fn get<T: ?Sized>(&self, re: Ref<T>) -> &T {
        self.try_get(re).unwrap()
    }

    /// Resolves an [`Own<T>`] into a reference, if the [`Own<T>`] was created in this arena.
    /// Otherwise returns [`DifferentArenaError`].
    ///
    /// Note that this requires a mutable reference to an [`Own<T>`], which is not [`Clone`], and
    /// therefore this cannot be used to obtain multiple mutable references to the same allocation.
    pub fn try_get_mut<T>(&self, own: &mut Own<T>) -> Result<&mut T, DifferentArenaError> {
        if own.arena_index == self.index {
            // SAFETY: The `if` statement checks that the pointer inside `own` belongs to this
            // arena, and since this arena is live, all pointers coming from it are live too.
            // The lifetime of the resulting reference is as long as the arena's lifetime, so the
            // reference cannot exist after the arena is dropped.
            Ok(unsafe { own.ptr.as_mut() })
        } else {
            Err(DifferentArenaError)
        }
    }

    /// Same as [`try_get_mut`][Self::try_get_mut], but panics on error.
    pub fn get_mut<T>(&self, re: &mut Own<T>) -> &mut T {
        self.try_get_mut(re).unwrap()
    }

    /// Resolves an [`OwnPinned<T>`] into a pinned reference, if the [`OwnPinned<T>`] was created in
    /// this arena. Otherwise returns [`DifferentArenaError`].
    ///
    /// Note that this requires a mutable reference to an [`OwnPinned<T>`], which is not [`Clone`],
    /// and therefore this cannot be used to obtain multiple mutable references to the same
    /// allocation.
    ///
    /// [`OwnPinned<T>`] needs to be a separate type from [`Own<T>`] because once an allocation
    /// becomes pinned, it must not be unpinned unless it implements [`Unpin`]. This is not the case
    /// with [`Own<T>`] as the references it returns may not be pinned.
    pub fn try_get_mut_pinned<T: ?Sized>(
        &self,
        own: &mut OwnPinned<T>,
    ) -> Result<Pin<&mut T>, DifferentArenaError> {
        if own.arena_index == self.index {
            // SAFETY: The `if` statement checks that the pointer inside `own` belongs to this
            // arena, and since this arena is live, all pointers coming from it are live too.
            // The lifetime of the resulting reference is as long as the arena's lifetime, so the
            // reference cannot exist after the arena is dropped.
            let mut_ref = unsafe { own.ptr.as_mut() };
            Ok(unsafe { Pin::new_unchecked(mut_ref) })
        } else {
            Err(DifferentArenaError)
        }
    }

    /// Same as [`try_get_mut`][Self::try_get_mut], but panics on error.
    pub fn get_mut_pinned<T: ?Sized>(&self, re: &mut OwnPinned<T>) -> Pin<&mut T> {
        self.try_get_mut_pinned(re).unwrap()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        let mut allocs = self.allocs.lock();
        let mut droppers = self.droppers.lock();

        for dropper in droppers.drain(..) {
            // SAFETY: `Arena` has ownership of the pointer and we can safely assume it has not been
            // dropped beforehand.
            unsafe { (dropper.drop_fn)(allocs[dropper.alloc_index]) };
        }

        for mut alloc in allocs.drain(..) {
            // SAFETY: `Arena` has ownership of the pointer and we can safely assume it's still
            // valid at this point.
            unsafe {
                let layout = alloc.as_mut().layout;
                std::alloc::dealloc(alloc.as_ptr().cast(), layout);
            }
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared reference to an element in an arena.
///
/// Unlike a regular reference, `Ref<T>` does not encode a lifetime. Instead it tracks the lifetime
/// of the owning arena dynamically by storing a unique ID of its owning arena. This allows for
/// circular references to the arena's contents.
///
/// Note that a `Ref<T>` still requires access to the owning arena to read what's in the reference.
#[derive(Debug)]
pub struct Ref<T: ?Sized> {
    ptr: NonNull<T>,
    arena_index: usize,
}

impl<T: ?Sized> Copy for Ref<T> {}

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Owned reference to an element in an arena.
///
/// This is similar to [`Ref<T>`], but it allows for accessing data mutably. Therefore there may
/// only exist one `Own<T>` per allocation. (`Own<T>` is not [`Clone`].)
///
/// An `Own<T>` can be downgraded to a [`Ref<T>`] once mutability is not needed, but this consumes
/// the `Own<T>.
#[derive(Debug)]
pub struct Own<T: ?Sized> {
    ptr: NonNull<T>,
    arena_index: usize,
}

impl<T> Own<T> {
    pub fn downgrade(self) -> Ref<T> {
        Ref {
            ptr: self.ptr,
            arena_index: self.arena_index,
        }
    }
}

/// Owned, *pinned* reference to an element in the arena.
///
/// This type is similar to [`Own<T>`], but it always returns a pinned mutable reference.
#[derive(Debug)]
pub struct OwnPinned<T: ?Sized> {
    ptr: NonNull<T>,
    arena_index: usize,
}

impl<'a, F, T> OwnPinned<F>
where
    F: Future<Output = T> + 'a,
{
    /// Coerces an `OwnPinned<F>` to an `OwnPinned<dyn Future>`.
    ///
    /// This is a hack to get around [`CoerceUnsized`][std::ops::CoerceUnsized] being unstable.
    pub fn as_dyn_future(self) -> OwnPinned<dyn Future<Output = T> + 'a> {
        OwnPinned {
            ptr: self.ptr,
            arena_index: self.arena_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DifferentArenaError;

impl fmt::Display for DifferentArenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("reference was allocated in a different arena")
    }
}

#[cfg(test)]
mod tests {
    use crate::arena::DifferentArenaError;

    use super::Arena;

    #[test]
    fn alloc() {
        let arena = Arena::new();
        let value = arena.alloc(1);
        *value = 2;
        let value: &i32 = value;
        let value2 = value;
        assert_eq!(value, value2);
    }

    #[test]
    fn shared() {
        let arena = Arena::new();
        let shared = arena.alloc_own(1).downgrade();
        let shared2 = shared;

        let i = arena.get(shared);
        let i2 = arena.get(shared2);
        assert_eq!(i, i2);

        let arena2 = Arena::new();
        assert_eq!(arena2.try_get(shared), Err(DifferentArenaError));
    }

    #[test]
    fn self_referencing() {
        let arena = Arena::new();
        let re = &*arena.alloc(1);
        arena.alloc(re);
        arena.alloc(&arena);
    }

    #[test]
    fn dropping() {
        let arena = Arena::new();
        arena.alloc(vec![1]);
    }

    #[test]
    fn zst() {
        let arena = Arena::new();
        let re = arena.alloc(());
        assert_eq!(re, &mut ());
    }
}
