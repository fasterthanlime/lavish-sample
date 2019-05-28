// This file is generated by lavish: DO NOT EDIT
// https://github.com/fasterthanlime/lavish

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(clippy::all)]
#![allow(unknown_lints)]
#![allow(unused)]

pub use __::*;

mod __ {
    // Notes: as of 2019-05-21, futures-preview is required
    use futures::prelude::*;
    use std::pin::Pin;
    use std::sync::Arc;
    
    use lavish_rpc as rpc;
    use rpc::{Atom, erased_serde, serde_derive::*};
    
    #[derive(Serialize, Debug)]
    #[serde(untagged)]
    #[allow(non_camel_case_types, unused)]
    pub enum Params {
        get_cookies(get_cookies::Params),
        get_user_agent(get_user_agent::Params),
        ping(ping::Params),
        ping_ping(ping::ping::Params),
    }
    
    #[derive(Serialize, Debug)]
    #[serde(untagged)]
    #[allow(non_camel_case_types, unused)]
    pub enum Results {
        get_cookies(get_cookies::Results),
        get_user_agent(get_user_agent::Results),
        ping(ping::Results),
        ping_ping(ping::ping::Results),
    }
    
    #[derive(Serialize, Debug)]
    #[serde(untagged)]
    #[allow(non_camel_case_types, unused)]
    pub enum NotificationParams {
    }
    
    pub type Message = rpc::Message<Params, NotificationParams, Results>;
    pub type Handle = rpc::Handle<Params, NotificationParams, Results>;
    pub type Protocol = rpc::Protocol<Params, NotificationParams, Results>;
    
    pub fn protocol() -> Protocol {
        Protocol::new()
    }
    
    impl rpc::Atom for Params {
        fn method(&self) -> &'static str {
            match self {
                Params::get_cookies(_) => "get_cookies",
                Params::get_user_agent(_) => "get_user_agent",
                Params::ping(_) => "ping",
                Params::ping_ping(_) => "ping.ping",
            }
        }
        
        fn deserialize(
            method: &str,
            de: &mut erased_serde::Deserializer,
        ) -> erased_serde::Result<Self> {
            use erased_serde::deserialize as deser;
            use serde::de::Error;
            
            match method {
                "get_cookies" =>
                    Ok(Params::get_cookies(deser::<get_cookies::Params>(de)?)),
                "get_user_agent" =>
                    Ok(Params::get_user_agent(deser::<get_user_agent::Params>(de)?)),
                "ping" =>
                    Ok(Params::ping(deser::<ping::Params>(de)?)),
                "ping.ping" =>
                    Ok(Params::ping_ping(deser::<ping::ping::Params>(de)?)),
                _ => Err(erased_serde::Error::custom(format!(
                    "unknown method: {}",
                    method,
                ))),
            }
        }
    }
    
    impl rpc::Atom for Results {
        fn method(&self) -> &'static str {
            match self {
                Results::get_cookies(_) => "get_cookies",
                Results::get_user_agent(_) => "get_user_agent",
                Results::ping(_) => "ping",
                Results::ping_ping(_) => "ping.ping",
            }
        }
        
        fn deserialize(
            method: &str,
            de: &mut erased_serde::Deserializer,
        ) -> erased_serde::Result<Self> {
            use erased_serde::deserialize as deser;
            use serde::de::Error;
            
            match method {
                "get_cookies" =>
                    Ok(Results::get_cookies(deser::<get_cookies::Results>(de)?)),
                "get_user_agent" =>
                    Ok(Results::get_user_agent(deser::<get_user_agent::Results>(de)?)),
                "ping" =>
                    Ok(Results::ping(deser::<ping::Results>(de)?)),
                "ping.ping" =>
                    Ok(Results::ping_ping(deser::<ping::ping::Results>(de)?)),
                _ => Err(erased_serde::Error::custom(format!(
                    "unknown method: {}",
                    method,
                ))),
            }
        }
    }
    
    impl rpc::Atom for NotificationParams {
        fn method(&self) -> &'static str {
            match self {
                _ => unimplemented!()
            }
        }
        
        fn deserialize(
            method: &str,
            de: &mut erased_serde::Deserializer,
        ) -> erased_serde::Result<Self> {
            use erased_serde::deserialize as deser;
            use serde::de::Error;
            
            match method {
                _ => Err(erased_serde::Error::custom(format!(
                    "unknown method: {}",
                    method,
                ))),
            }
        }
    }
    
    pub struct Call<T, PP> {
        pub state: Arc<T>,
        pub handle: Handle,
        pub params: PP,
    }
    
    pub type SlotFuture = 
        Future<Output = Result<Results, rpc::Error>> + Send + 'static;
    
    pub type SlotReturn = Pin<Box<SlotFuture>>;
    
    pub type SlotFn<T> = 
        Fn(Arc<T>, Handle, Params) -> SlotReturn + 'static + Send + Sync;
    
    pub type Slot<T> = Option<Box<SlotFn<T>>>;
    
    pub struct Handler<T> {
        state: Arc<T>,
        get_cookies: Slot<T>,
        get_user_agent: Slot<T>,
        ping: Slot<T>,
        ping_ping: Slot<T>,
    }
    
    impl<T> Handler<T> {
        pub fn new(state: Arc<T>) -> Self {
            Self {
                state,
                get_cookies: None,
                get_user_agent: None,
                ping: None,
                ping_ping: None,
            }
        }
        
        pub fn on_get_cookies<F, FT> (&mut self, f: F)
        where
            F: Fn(Call<T, get_cookies::Params>) -> FT + Sync + Send + 'static,
            FT: Future<Output = Result<get_cookies::Results, lavish_rpc::Error>> + Send + 'static,
        {
            self.get_cookies = Some(Box::new(move |state, handle, params| {
                Box::pin(
                    f(Call {
                        state, handle,
                        params: get_cookies::Params::downgrade(params).unwrap(),
                    }).map_ok(Results::get_cookies)
                )
            }));
        }
        
        pub fn on_get_user_agent<F, FT> (&mut self, f: F)
        where
            F: Fn(Call<T, get_user_agent::Params>) -> FT + Sync + Send + 'static,
            FT: Future<Output = Result<get_user_agent::Results, lavish_rpc::Error>> + Send + 'static,
        {
            self.get_user_agent = Some(Box::new(move |state, handle, params| {
                Box::pin(
                    f(Call {
                        state, handle,
                        params: get_user_agent::Params::downgrade(params).unwrap(),
                    }).map_ok(Results::get_user_agent)
                )
            }));
        }
        
        pub fn on_ping<F, FT> (&mut self, f: F)
        where
            F: Fn(Call<T, ping::Params>) -> FT + Sync + Send + 'static,
            FT: Future<Output = Result<ping::Results, lavish_rpc::Error>> + Send + 'static,
        {
            self.ping = Some(Box::new(move |state, handle, params| {
                Box::pin(
                    f(Call {
                        state, handle,
                        params: ping::Params::downgrade(params).unwrap(),
                    }).map_ok(|_| Results::ping(ping::Results {}))
                )
            }));
        }
        
        pub fn on_ping_ping<F, FT> (&mut self, f: F)
        where
            F: Fn(Call<T, ping::ping::Params>) -> FT + Sync + Send + 'static,
            FT: Future<Output = Result<ping::ping::Results, lavish_rpc::Error>> + Send + 'static,
        {
            self.ping_ping = Some(Box::new(move |state, handle, params| {
                Box::pin(
                    f(Call {
                        state, handle,
                        params: ping::ping::Params::downgrade(params).unwrap(),
                    }).map_ok(|_| Results::ping_ping(ping::ping::Results {}))
                )
            }));
        }
        
    }
    
    type HandlerRet = Pin<Box<dyn Future<Output = Result<Results, rpc::Error>> + Send + 'static>>;
    
    impl<T> rpc::Handler<Params, NotificationParams, Results, HandlerRet> for Handler<T>
    where
        T: Send + Sync,
    {
        fn handle(&self, handle: Handle, params: Params) -> HandlerRet {
            let method = params.method();
            let slot = match params {
                Params::get_cookies(_) => self.get_cookies.as_ref(),
                Params::get_user_agent(_) => self.get_user_agent.as_ref(),
                Params::ping(_) => self.ping.as_ref(),
                Params::ping_ping(_) => self.ping_ping.as_ref(),
                _ => None,
            };
            match slot {
                Some(slot_fn) => {
                    let res = slot_fn(self.state.clone(), handle, params);
                    Box::pin(async move { Ok(res.await?) })
                }
                None => Box::pin(async move { Err(rpc::Error::MethodUnimplemented(method)) }),
            }
        }
    }
    
    use lavish_rpc::serde_derive::*;
    
    /// A key/value pair used to remember session information.
    /// 
    /// Can be harmful in real life, but this is just a sample schema
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Cookie {
        /// The key of the cookie
        pub key: String,
        /// The value of the cookie.
        /// Although it's typed as a string, it can be anything underneath.
        pub value: String,
    }
    
    /// Ask for a list of cookies from the server.
    pub mod get_cookies {
        use futures::prelude::*;
        use lavish_rpc::serde_derive::*;
        use super::super::__;
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Params {
        }
        
        impl Params {
            pub fn downgrade(p: __::Params) -> Option<Self> {
                match p {
                    __::Params::get_cookies(p) => Some(p),
                    _ => None,
                }
            }
        }
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Results {
            pub cookies: Vec<super::Cookie>,
        }
        
        impl Results {
            pub fn downgrade(p: __::Results) -> Option<Self> {
                match p {
                    __::Results::get_cookies(p) => Some(p),
                    _ => None,
                }
            }
        }
        
        pub async fn call(h: &__::Handle, p: ()) -> Result<Results, lavish_rpc::Error> {
            h.call(
                __::Params::get_cookies(Params {}),
                Results::downgrade,
            ).await
        }
        }
        
    /// Ask the client what its user-agent is.
    pub mod get_user_agent {
        use futures::prelude::*;
        use lavish_rpc::serde_derive::*;
        use super::super::__;
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Params {
        }
        
        impl Params {
            pub fn downgrade(p: __::Params) -> Option<Self> {
                match p {
                    __::Params::get_user_agent(p) => Some(p),
                    _ => None,
                }
            }
        }
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Results {
            pub user_agent: String,
        }
        
        impl Results {
            pub fn downgrade(p: __::Results) -> Option<Self> {
                match p {
                    __::Results::get_user_agent(p) => Some(p),
                    _ => None,
                }
            }
        }
        
        pub async fn call(h: &__::Handle, p: ()) -> Result<Results, lavish_rpc::Error> {
            h.call(
                __::Params::get_user_agent(Params {}),
                Results::downgrade,
            ).await
        }
        }
        
    /// Ping the server to make sure it's alive
    pub mod ping {
        use futures::prelude::*;
        use lavish_rpc::serde_derive::*;
        use super::super::__;
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Params {
        }
        
        impl Params {
            pub fn downgrade(p: __::Params) -> Option<Self> {
                match p {
                    __::Params::ping(p) => Some(p),
                    _ => None,
                }
            }
        }
        
        #[derive(Serialize, Deserialize, Debug)]
        pub struct Results {
        }
        
        impl Results {
            pub fn downgrade(p: __::Results) -> Option<Self> {
                match p {
                    __::Results::ping(p) => Some(p),
                    _ => None,
                }
            }
        }
        
        pub async fn call(h: &__::Handle, p: ()) -> Result<Results, lavish_rpc::Error> {
            h.call(
                __::Params::ping(Params {}),
                Results::downgrade,
            ).await
        }
        use lavish_rpc::serde_derive::*;
        
        /// Ping the client to make sure it's alive
        pub mod ping {
            use futures::prelude::*;
            use lavish_rpc::serde_derive::*;
            use super::super::super::__;
            
            #[derive(Serialize, Deserialize, Debug)]
            pub struct Params {
            }
            
            impl Params {
                pub fn downgrade(p: __::Params) -> Option<Self> {
                    match p {
                        __::Params::ping_ping(p) => Some(p),
                        _ => None,
                    }
                }
            }
            
            #[derive(Serialize, Deserialize, Debug)]
            pub struct Results {
            }
            
            impl Results {
                pub fn downgrade(p: __::Results) -> Option<Self> {
                    match p {
                        __::Results::ping_ping(p) => Some(p),
                        _ => None,
                    }
                }
            }
            
            pub async fn call(h: &__::Handle, p: ()) -> Result<Results, lavish_rpc::Error> {
                h.call(
                    __::Params::ping_ping(Params {}),
                    Results::downgrade,
                ).await
            }
            }
            
        }
        
    
    pub struct PeerBuilder<C>
    where
        C: lavish_rpc::Conn,
    {
        conn: C,
        pool: futures::executor::ThreadPool,
    }
    
    impl<C> PeerBuilder<C>
    where
        C: lavish_rpc::Conn,
    {
        pub fn new(conn: C, pool: futures::executor::ThreadPool) -> Self {
            Self { conn, pool }
        }
        
        pub fn with_noop_handler(self) -> Result<Handle, lavish_rpc::Error> {
            self.with_handler(|_| {})
        }
        
        pub fn with_handler<S>(self, setup: S) -> Result<Handle, lavish_rpc::Error>
        where
            S: Fn(&mut Handler<()>),
        {
            self.with_stateful_handler(std::sync::Arc::new(()), setup)
        }
        
        pub fn with_stateful_handler<T, S>(self, state: Arc<T>, setup: S) -> Result<Handle, lavish_rpc::Error>
        where
            S: Fn(&mut Handler<T>),
            T: Sync + Send + 'static,
        {
            let mut handler = Handler::new(state);
            setup(&mut handler);
            lavish_rpc::connect(protocol(), handler, self.conn, self.pool)
        }
    }
    
    pub fn peer<C>(conn: C, pool: futures::executor::ThreadPool) -> PeerBuilder<C>
    where
        C: lavish_rpc::Conn,
    {
        PeerBuilder::new(conn, pool)
    }
}
