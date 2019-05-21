use std::pin::Pin;
use std::sync::Arc;

use futures::prelude::*;

use lavish_rpc::{Atom, Handler};

use super::proto::{self, Call, Handle, MethodHandler};

pub struct PluggableHandler<'a, T> {
    state: Arc<T>,
    double_util_Print: MethodHandler<
        'a,
        T,
        proto::double::util::print::Params,
        proto::double::util::print::Results,
    >,
}

impl<'a, T> PluggableHandler<'a, T>
where
    T: Send + Sync,
{
    pub fn new(state: T) -> Self {
        Self {
            state: Arc::new(state),
            double_util_Print: None,
        }
    }

    pub fn on_double_util_Print<F, FT>(&mut self, f: F)
    where
        F: Fn(Call<T, proto::double::util::print::Params>) -> FT + Sync + Send + 'a,
        FT: Future<Output = Result<proto::double::util::print::Results, lavish_rpc::Error>>
            + Send
            + 'static,
    {
        self.double_util_Print = Some(Box::new(move |call| Box::pin(f(call))))
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
        match params {
            proto::Params::double_util_Print(params) => match self.double_util_Print.as_ref() {
                Some(hm) => {
                    let call = Call {
                        state: self.state.clone(),
                        handle,
                        params,
                    };
                    let res = hm(call);
                    Box::pin(async move { Ok(proto::Results::double_util_Print(res.await?)) })
                }
                None => {
                    Box::pin(async move { Err(lavish_rpc::Error::MethodUnimplemented(method)) })
                }
            },
            _ => Box::pin(async move { Err(lavish_rpc::Error::MethodUnimplemented(method)) }),
        }
    }
}
