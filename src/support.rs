use std::pin::Pin;
use std::sync::Arc;

use futures::prelude::*;

use lavish_rpc as rpc;
use lavish_rpc::{Atom, Handler};

use super::proto::{self, Handle};

pub type Call<T, PP> = rpc::Call<T, proto::Params, proto::NotificationParams, proto::Results, PP>;

pub type MethodHandler<'a, T> = Option<
    Box<
        Fn(
                Arc<T>,
                Handle,
                proto::Params,
            )
                -> (Pin<Box<Future<Output = Result<proto::Results, rpc::Error>> + Send + 'static>>)
            + 'a
            + Send
            + Sync,
    >,
>;

pub struct PluggableHandler<'a, T> {
    state: Arc<T>,
    pub double_util_print: MethodHandler<'a, T>,
}

impl<'a, T> PluggableHandler<'a, T>
where
    T: Send + Sync + 'static,
{
    pub fn new(state: T) -> Self {
        Self {
            state: Arc::new(state),
            double_util_print: None,
        }
    }

    pub fn on_double_util_print<F, FT>(&mut self, f: F)
    where
        F: Fn(Call<T, proto::double::util::print::Params>) -> FT + Sync + Send + 'static,
        FT: Future<Output = Result<proto::double::util::print::Results, lavish_rpc::Error>>
            + Send
            + 'static,
    {
        self.double_util_print = Some(Box::new(move |state, handle, params| {
            Box::pin(
                f(Call {
                    state,
                    handle,
                    params: proto::double::util::print::Params::downgrade(params).unwrap(),
                })
                .map_ok(proto::Results::double_util_print),
            )
        }));
    }
}

type HandlerRet =
    Pin<Box<dyn Future<Output = Result<proto::Results, lavish_rpc::Error>> + Send + 'static>>;

impl<'a, T> Handler<proto::Params, proto::NotificationParams, proto::Results, HandlerRet>
    for PluggableHandler<'a, T>
where
    T: Send + Sync,
{
    fn handle(&self, handle: Handle, params: proto::Params) -> HandlerRet {
        let method = params.method();
        let hm = match params {
            proto::Params::double_util_print(_) => self.double_util_print.as_ref(),
            _ => None,
        };
        match hm {
            Some(hm) => {
                let res = hm(self.state.clone(), handle, params);
                Box::pin(async move { Ok(res.await?) })
            }
            None => Box::pin(async move { Err(lavish_rpc::Error::MethodUnimplemented(method)) }),
        }
    }
}
