use std::future::Future;
use std::panic::RefUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;
use crate::borrow_bag::{BorrowBag, Handle, Lookup};
use crate::pipeline::processing_context::State;
use anyhow::Result;
// use futures_util::{future, FutureExt};

pub type PipelineSet<P> = Arc<BorrowBag<P>>;

pub type EditablePipelineSet<P> = BorrowBag<P>;

pub fn new_pipeline_set() -> EditablePipelineSet<()> {
    BorrowBag::new()
}

pub fn finalize_pipeline_set<P>(eps: EditablePipelineSet<P>) -> PipelineSet<P> {
    Arc::new(eps)
}

pub type HandlerFuture = dyn Future<Output = ()> + Send;

pub trait Middleware {
    fn call<Chain>(self, state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
        where
            Chain: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
            Self: Sized;
}

pub unsafe trait MiddlewareChain: Sized {
    fn call<F>(self, state: State, f: F) -> Pin<Box<HandlerFuture>>
        where
            F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static;
}

unsafe impl MiddlewareChain for () {
    fn call<F>(self, state: State, f: F) -> Pin<Box<HandlerFuture>>
        where
            F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        f(state)
    }
}

unsafe impl<T, U> MiddlewareChain for (T, U)
    where
        T: Middleware + Send + 'static,
        U: MiddlewareChain,
{
    fn call<F>(self, state: State, f: F) -> Pin<Box<HandlerFuture>>
        where
            F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        let (m, p) = self;
        // 递归调用
        p.call(state, move |state| m.call(state, f))
    }
}

pub trait NewMiddleware: Sync + RefUnwindSafe {
    type Instance: Middleware;
    fn new_middleware(&self) -> Result<Self::Instance>;
}

#[doc(hidden)]
pub unsafe trait NewMiddlewareChain: RefUnwindSafe + Sized {
    type Instance: MiddlewareChain;

    fn construct(&self) -> Result<Self::Instance>;
}

unsafe impl<T, U> NewMiddlewareChain for (T, U)
    where
        T: NewMiddleware,
        T::Instance: Send + 'static,
        U: NewMiddlewareChain,
{
    type Instance = (T::Instance, U::Instance);

    fn construct(&self) -> Result<Self::Instance> {
        let (ref nm, ref tail) = *self;
        Ok((nm.new_middleware()?, tail.construct()?))
    }
}

unsafe impl NewMiddlewareChain for () {
    type Instance = ();

    fn construct(&self) -> anyhow::Result<Self::Instance> {
        // trace!(" completed middleware pipeline construction");
        Ok(())
    }
}



pub struct Pipeline<T>
    where
        T: NewMiddlewareChain,
{
    chain: T,
}
struct PipelineInstance<T>
    where
        T: MiddlewareChain,
{
    chain: T,
}

impl<T> Pipeline<T>
    where
        T: NewMiddlewareChain,
{
    fn construct(&self) -> Result<PipelineInstance<T::Instance>> {
        Ok(PipelineInstance {
            chain: self.chain.construct()?,
        })
    }
}

impl<T> PipelineInstance<T>
    where
        T: MiddlewareChain,
{
    fn call<F>(self, state: State, f: F) -> Pin<Box<HandlerFuture>>
        where
            F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        self.chain.call(state, f)
    }
}



pub trait PipelineHandleChain<P>: RefUnwindSafe {
    fn call<F>(&self, pipelines: &PipelineSet<P>, state: State, f: F) -> Pin<Box<HandlerFuture>>
        where
            F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static;
}

pub struct PipelineBuilder<T>
    where
        T: NewMiddlewareChain,
{
    t: T,
}
impl<T> PipelineBuilder<T>
    where
        T: NewMiddlewareChain,
{
    pub fn build(self) -> Pipeline<T>
        where
            T: NewMiddlewareChain,
    {
        Pipeline { chain: self.t }
    }

    pub fn add<M>(self, m: M) -> PipelineBuilder<(M, T)>
        where
            M: NewMiddleware,
            M::Instance: Send + 'static,
            Self: Sized,
    {
        PipelineBuilder { t: (m, self.t) }
    }
}
pub fn new_pipeline() -> PipelineBuilder<()> {
    PipelineBuilder { t: () }
}



// impl<P, T, N, U> PipelineHandleChain<P> for (Handle<Pipeline<T>, N>, U)
//     where
//         T: NewMiddlewareChain,
//         T::Instance: Send + 'static,
//         U: PipelineHandleChain<P>,
//         P: Lookup<Pipeline<T>, N>,
//         N: RefUnwindSafe,
// {
//     fn call<F>(&self, pipelines: &PipelineSet<P>, state: State, f: F) -> Pin<Box<HandlerFuture>>
//         where
//             F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
//     {
//         let (handle, ref chain) = *self;
//         match pipelines.borrow(handle).construct() {
//             Ok(p) => chain.call(pipelines, state, move |state| p.call(state, f)),
//             Err(e) => {
//                 // trace!("[{}] error borrowing pipeline", request_id(&state));
//                 future::err((state, e.into())).boxed()
//             }
//         }
//     }
// }

/// The marker for the end of a `PipelineHandleChain`.
impl<P> PipelineHandleChain<P> for () {
    fn call<F>(&self, _: &PipelineSet<P>, state: State, f: F) -> Pin<Box<HandlerFuture>>
        where
            F: FnOnce(State) -> Pin<Box<HandlerFuture>> + Send + 'static,
    {
        // trace!("[{}] start pipeline", request_id(&state));
        f(state)
    }
}
