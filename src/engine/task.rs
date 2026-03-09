//! Async support for transition sequencing.
//!
//! [`TransactionRef`] implements [`std::future::IntoFuture`],
//! so every `set_*` call on a [`Layer`] is directly `await`-able.
//! Use any async executor — e.g. `tokio::spawn` — to sequence transitions without nested
//! [`on_finish`](crate::engine::TransactionRef::on_finish) closures:
//!
//! ```rust,ignore
//! tokio::spawn(async move {
//!     layer.set_position((100.0, 0.0), Transition::ease_in(0.4)).await;
//!     layer.set_opacity(0.0, Transition::ease_out(0.3)).await;
//! });
//! ```

use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
};

use super::{AnimationRef, TransactionRef};
use crate::layers::layer::Layer;

struct TransitionFutureState {
    finished: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

/// A future that resolves when a scheduled transition finishes.
///
/// Obtain one by `.await`-ing any value returned from a `set_*` method on a [`Layer`]:
///
/// ```rust,ignore
/// layer.set_position((200.0, 0.0), Transition::ease_in(0.5)).await;
/// ```
pub struct TransitionFuture {
    state: Arc<TransitionFutureState>,
    transaction_ref: TransactionRef,
}

impl Future for TransitionFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.finished.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        // Keep the waker fresh in case the executor changed.
        *self.state.waker.lock().unwrap() = Some(cx.waker().clone());

        // Re-check after storing the waker: the on_finish callback may have fired
        // in the window between the first check and now and found no waker to call.
        if self.state.finished.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        // Defensive check: the transaction may have been cleaned up before
        // into_future() was called (stale TransactionRef), in which case the
        // eagerly-registered callback will never fire. Resolve immediately.
        if self
            .transaction_ref
            .engine()
            .get_transaction(self.transaction_ref)
            .is_none()
        {
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

impl std::future::IntoFuture for TransactionRef {
    type Output = ();
    type IntoFuture = TransitionFuture;

    fn into_future(self) -> TransitionFuture {
        let state = Arc::new(TransitionFutureState {
            finished: AtomicBool::new(false),
            waker: Mutex::new(None),
        });
        // Register the on_finish callback eagerly so that a completion that
        // occurs between into_future() and the first poll() sets `finished`
        // before poll() runs, allowing it to return Ready immediately.
        let state_clone = Arc::clone(&state);
        self.on_finish(
            move |_layer: &Layer, _| {
                state_clone.finished.store(true, Ordering::Release);
                if let Some(waker) = state_clone.waker.lock().unwrap().take() {
                    waker.wake();
                }
            },
            true,
        );
        TransitionFuture {
            state,
            transaction_ref: self,
        }
    }
}

/// A future that resolves when a scheduled animation finishes.
///
/// Obtain one by `.await`-ing an [`AnimationRef`]:
///
/// ```rust,ignore
/// let anim = engine.add_animation_from_transition(&transition, true);
/// anim.await;
/// ```
pub struct AnimationFuture {
    state: Arc<TransitionFutureState>,
    animation_ref: AnimationRef,
}

impl Future for AnimationFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.finished.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        // Keep the waker fresh in case the executor changed.
        *self.state.waker.lock().unwrap() = Some(cx.waker().clone());

        // Re-check after storing the waker: the on_finish callback may have fired
        // in the window between the first check and now and found no waker to call.
        if self.state.finished.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        // Defensive check: the animation may have been cleaned up before
        // into_future() was called (stale AnimationRef), in which case the
        // eagerly-registered callback will never fire. Resolve immediately.
        if self
            .animation_ref
            .engine()
            .get_animation(self.animation_ref)
            .is_none()
        {
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

impl std::future::IntoFuture for AnimationRef {
    type Output = ();
    type IntoFuture = AnimationFuture;

    fn into_future(self) -> AnimationFuture {
        let state = Arc::new(TransitionFutureState {
            finished: AtomicBool::new(false),
            waker: Mutex::new(None),
        });
        // Register the on_finish callback eagerly so that a completion that
        // occurs between into_future() and the first poll() sets `finished`
        // before poll() runs, allowing it to return Ready immediately.
        let state_clone = Arc::clone(&state);
        self.on_finish(
            move |_progress: f32| {
                state_clone.finished.store(true, Ordering::Release);
                if let Some(waker) = state_clone.waker.lock().unwrap().take() {
                    waker.wake();
                }
            },
            true,
        );
        AnimationFuture {
            state,
            animation_ref: self,
        }
    }
}
