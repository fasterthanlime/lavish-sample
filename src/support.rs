use std::pin::Pin;
use std::sync::Arc;

use futures::prelude::*;

use lavish_rpc::{Atom, Handle, Handler, Protocol};

use super::proto;

pub fn protocol() -> Protocol<proto::Params, proto::NotificationParams, proto::Results> {
    Protocol::new()
}

type ProtoHandle = Handle<proto::Params, proto::NotificationParams, proto::Results>;

pub struct Call<T, P> {
    pub state: Arc<T>,
    pub handle: ProtoHandle,
    pub params: P,
}

type MethodHandler<'a, T, P, R> = Option<
    Box<
        (Fn(Call<T, P>) -> (Pin<Box<Future<Output = Result<R, String>> + Send + 'static>>))
            + Sync
            + Send
            + 'a,
    >,
>;

pub struct PluggableHandler<'a, T> {
    state: Arc<T>,
    double_print: MethodHandler<'a, T, proto::double::print::Params, proto::double::print::Results>,
}

impl<'a, T> PluggableHandler<'a, T>
where
    T: Send + Sync,
{
    pub fn new(state: T) -> Self {
        Self {
            state: Arc::new(state),
            double_print: None,
        }
    }

    pub fn on_double_print<F, FT>(&mut self, f: F)
    where
        F: Fn(Call<T, proto::double::print::Params>) -> FT + Sync + Send + 'a,
        FT: Future<Output = Result<proto::double::print::Results, String>> + Send + 'static,
    {
        self.double_print = Some(Box::new(move |call| Box::pin(f(call))))
    }
}

type HandlerRet = Pin<Box<dyn Future<Output = Result<proto::Results, String>> + Send + 'static>>;

impl<'a, T> Handler<proto::Params, proto::NotificationParams, proto::Results, HandlerRet>
    for PluggableHandler<'a, T>
where
    T: Send + Sync,
{
    fn handle(
        &self,
        handle: Handle<proto::Params, proto::NotificationParams, proto::Results>,
        params: proto::Params,
    ) -> HandlerRet {
        let method = params.method();
        match params {
            proto::Params::double_Print(params) => match self.double_print.as_ref() {
                Some(hm) => {
                    let call = Call {
                        state: self.state.clone(),
                        handle,
                        params,
                    };
                    let res = hm(call);
                    Box::pin(async move { Ok(proto::Results::double_Print(res.await?)) })
                }
                None => Box::pin(async move { Err(format!("no handler for {}", method)) }),
            },
            _ => Box::pin(async move { Err(format!("no handler for {}", method)) }),
        }
    }
}