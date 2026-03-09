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
    registered: bool,
}

impl Future for TransitionFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.finished.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        // Keep the waker fresh in case the executor changed.
        *self.state.waker.lock().unwrap() = Some(cx.waker().clone());

        // Register the on_finish callback exactly once.
        if !self.registered {
            let state = Arc::clone(&self.state);
            self.transaction_ref.on_finish(
                move |_layer: &Layer, _| {
                    state.finished.store(true, Ordering::Release);
                    if let Some(waker) = state.waker.lock().unwrap().take() {
                        waker.wake();
                    }
                },
                true,
            );
            self.registered = true;
        }

        Poll::Pending
    }
}

impl std::future::IntoFuture for TransactionRef {
    type Output = ();
    type IntoFuture = TransitionFuture;

    fn into_future(self) -> TransitionFuture {
        TransitionFuture {
            state: Arc::new(TransitionFutureState {
                finished: AtomicBool::new(false),
                waker: Mutex::new(None),
            }),
            transaction_ref: self,
            registered: false,
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
    registered: bool,
}

impl Future for AnimationFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.finished.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        // Keep the waker fresh in case the executor changed.
        *self.state.waker.lock().unwrap() = Some(cx.waker().clone());

        // Register the on_finish callback exactly once.
        if !self.registered {
            let state = Arc::clone(&self.state);
            self.animation_ref.on_finish(
                move |_progress: f32| {
                    state.finished.store(true, Ordering::Release);
                    if let Some(waker) = state.waker.lock().unwrap().take() {
                        waker.wake();
                    }
                },
                true,
            );
            self.registered = true;
        }

        Poll::Pending
    }
}

impl std::future::IntoFuture for AnimationRef {
    type Output = ();
    type IntoFuture = AnimationFuture;

    fn into_future(self) -> AnimationFuture {
        AnimationFuture {
            state: Arc::new(TransitionFutureState {
                finished: AtomicBool::new(false),
                waker: Mutex::new(None),
            }),
            animation_ref: self,
            registered: false,
        }
    }
}
