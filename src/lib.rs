//! Main library entry point for openapi_client implementation.
extern crate chrono;
extern crate futures;
extern crate i2cbus_api;
extern crate pca9956b_api;
//#[macro_use]
extern crate swagger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate hyper;
extern crate tokio_core;
extern crate uuid;
#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate log;

mod http;
mod server;

mod errors {
    error_chain! {}
}

pub use crate::http::get_env;
pub use self::errors::*;
use std::clone::Clone;
use std::io;
use std::marker::PhantomData;
use swagger::{Has, XSpanIdString};

pub struct NewService<C> {
    marker: PhantomData<C>,
}

impl<C> NewService<C> {
    pub fn new() -> Self {
        NewService {
            marker: PhantomData,
        }
    }
}

impl<C> hyper::server::NewService for NewService<C>
where
    C: Has<XSpanIdString> + Clone + 'static,
{
    type Request = (hyper::Request, C);
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = pca9956b_api::server::Service<server::Server<C>, C>;

    /// Instantiate a new server.
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(pca9956b_api::server::Service::new(server::Server::new()))
    }
}
