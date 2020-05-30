//! Main library entry point for pca9956b_api implementation.

#![allow(unused_imports)]

mod errors {
    error_chain::error_chain!{}
}

pub use self::errors::*;

use chrono;
use futures::{future, Future, Stream};
use hyper::server::conn::Http;
use hyper::service::MakeService as _;
use log::{info, warn};
use openssl::ssl::SslAcceptorBuilder;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use swagger;
use swagger::{Has, XSpanIdString};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use tokio::net::TcpListener;


#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use tokio_openssl::SslAcceptorExt;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use pca9956b_api::models;

mod http;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub fn create(addr: &str, ssl: Option<SslAcceptorBuilder>) -> Box<dyn Future<Item = (), Error = ()> + Send> {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service_fn = MakeService::new(server);

    let service_fn = MakeAllowAllAuthenticator::new(service_fn, "cosmo");

    let service_fn =
        pca9956b_api::server::context::MakeAddContext::<_, EmptyContext>::new(
            service_fn
        );

    match ssl {
        Some(ssl) => {
            let tls_acceptor = ssl.build();
            let service_fn = Arc::new(Mutex::new(service_fn));
            let tls_listener = TcpListener::bind(&addr).unwrap().incoming().for_each(move |tcp| {
                let addr = tcp.peer_addr().expect("Unable to get remote address");

                let service_fn = service_fn.clone();

                hyper::rt::spawn(tls_acceptor.accept_async(tcp).map_err(|_| ()).and_then(move |tls| {
                    let ms = {
                        let mut service_fn = service_fn.lock().unwrap();
                        service_fn.make_service(&addr)
                    };

                    ms.and_then(move |service| {
                        Http::new().serve_connection(tls, service)
                    }).map_err(|_| ())
                }));

                Ok(())
            }).map_err(|_| ());

            Box::new(tls_listener)
        },
        None => Box::new(hyper::server::Server::bind(&addr).serve(service_fn).map_err(|e| panic!("{:?}", e))),
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server{marker: PhantomData}
    }
}

use pca9956b_api::{
    Api,
    ApiError,
    ClearErrorResponse,
    GetAddrEnabledResponse,
    GetAddrInfoResponse,
    GetAddrValueResponse,
    GetApiResponse,
    GetConfigResponse,
    GetCurrentResponse,
    GetErrorResponse,
    GetErrorsResponse,
    GetFreqResponse,
    GetGroupResponse,
    GetLedCurrentResponse,
    GetLedErrorResponse,
    GetLedInfoResponse,
    GetLedInfoAllResponse,
    GetLedPwmResponse,
    GetLedStateResponse,
    GetOffsetResponse,
    GetOutputChangeResponse,
    GetOverTempResponse,
    GetPwmResponse,
    GetSleepResponse,
    ResetResponse,
    SetAddrEnabledResponse,
    SetAddrValueResponse,
    SetConfigResponse,
    SetCurrentResponse,
    SetFreqResponse,
    SetGroupResponse,
    SetLedCurrentResponse,
    SetLedErrorResponse,
    SetLedInfoResponse,
    SetLedInfoAllResponse,
    SetLedPwmResponse,
    SetLedStateResponse,
    SetOffsetResponse,
    SetOutputChangeResponse,
    SetPwmResponse,
    SetSleepResponse,
};
use pca9956b_api::server::MakeService;

impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString>,
{
    fn clear_error(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = ClearErrorResponse, Error = ApiError> + Send> {
        http::clear_error(bus_id.into(), addr.into(), true.into())
    }

    fn get_addr_enabled(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetAddrEnabledResponse, Error = ApiError> + Send> {
        http::get_addr_enabled(bus_id.into(), addr.into(), num.into())
    }

    fn get_addr_info(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetAddrInfoResponse, Error = ApiError> + Send> {
        http::get_addr_info(bus_id.into(), addr.into(), num.into())
    }

    fn get_addr_value(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetAddrValueResponse, Error = ApiError> + Send> {
        http::get_addr_value(bus_id.into(), addr.into(), num.into())
    }

    fn get_api(&self, _context: &C) -> Box<dyn Future<Item = GetApiResponse, Error = ApiError> + Send> {
        http::get_api()
    }

    fn get_config(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetConfigResponse, Error = ApiError> + Send> {
        info!(
            "get_config({}, {})",
            bus_id,
            addr,
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_current(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetCurrentResponse, Error = ApiError> + Send> {
        http::get_current(bus_id.into(), addr.into())
    }

    fn get_error(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetErrorResponse, Error = ApiError> + Send> {
        http::get_error(bus_id.into(), addr.into())
    }

    fn get_errors(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetErrorsResponse, Error = ApiError> + Send> {
        info!(
            "get_errors({}, {})",
            bus_id,
            addr,
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_freq(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetFreqResponse, Error = ApiError> + Send> {
        http::get_freq(bus_id.into(), addr.into())
    }

    fn get_group(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetGroupResponse, Error = ApiError> + Send> {
        http::get_group(bus_id.into(), addr.into())
    }

    fn get_led_current(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetLedCurrentResponse, Error = ApiError> + Send> {
        http::get_led_current(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_error(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetLedErrorResponse, Error = ApiError> + Send> {
        http::get_led_error(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_info(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetLedInfoResponse, Error = ApiError> + Send> {
        http::get_led_info(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_info_all(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetLedInfoAllResponse, Error = ApiError> + Send> {
        info!(
            "get_led_info_all({}, {})",
            bus_id,
            addr,
        );
        http::get_led_info_all(bus_id.into(), addr.into())
    }

    fn get_led_pwm(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetLedPwmResponse, Error = ApiError> + Send> {
        http::get_led_pwm(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_state(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetLedStateResponse, Error = ApiError> + Send> {
        http::get_led_state(bus_id.into(), addr.into(), led.into())
    }

    fn get_offset(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetOffsetResponse, Error = ApiError> + Send> {

        http::get_offset(bus_id.into(), addr.into())
    }

    fn get_output_change(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetOutputChangeResponse, Error = ApiError> + Send> {
        http::get_output_change(bus_id.into(), addr.into())
    }

    fn get_over_temp(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetOverTempResponse, Error = ApiError> + Send> {
        info!(
            "get_over_temp({}, {})",
            bus_id,
            addr,
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_pwm(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetPwmResponse, Error = ApiError> + Send> {
        http::get_pwm(bus_id.into(), addr.into())
    }

    fn get_sleep(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = GetSleepResponse, Error = ApiError> + Send> {
        http::get_sleep(bus_id.into(), addr.into())
    }

    fn reset(
        &self,
        bus_id: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = ResetResponse, Error = ApiError> + Send> {
        http::reset(bus_id.into())
    }

    fn set_addr_enabled(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        enabled: bool,
        _context: &C,
    ) -> Box<dyn Future<Item = SetAddrEnabledResponse, Error = ApiError> + Send> {
        http::set_addr_enabled(bus_id.into(), addr.into(), num.into(), enabled.into())
    }

    fn set_addr_value(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        addr_val: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetAddrValueResponse, Error = ApiError> + Send> {
        http::set_addr_value(bus_id.into(), addr.into(), num.into(), addr_val.into())
    }

    fn set_config(
        &self,
        bus_id: i32,
        addr: i32,
        config: models::Config,
        _context: &C,
    ) -> Box<dyn Future<Item = SetConfigResponse, Error = ApiError> + Send> {
        info!(
            "set_config({}, {}, {:?})",
            bus_id,
            addr,
            config,
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn set_current(
        &self,
        bus_id: i32,
        addr: i32,
        current: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetCurrentResponse, Error = ApiError> + Send> {
        http::set_current(bus_id.into(), addr.into(), current.into())
    }

    fn set_freq(
        &self,
        bus_id: i32,
        addr: i32,
        freq: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetFreqResponse, Error = ApiError> + Send> {
        http::set_freq(bus_id.into(), addr.into(), freq.into())
    }

    fn set_group(
        &self,
        bus_id: i32,
        addr: i32,
        group: models::Group,
        _context: &C,
    ) -> Box<dyn Future<Item = SetGroupResponse, Error = ApiError> + Send> {
        http::set_group(bus_id.into(), addr.into(), group.into())
    }

    fn set_led_current(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        current: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetLedCurrentResponse, Error = ApiError> + Send> {
        http::set_led_current(bus_id.into(), addr.into(), led.into(), current.into())
    }

    fn set_led_error(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        error: models::LedError,
        _context: &C,
    ) -> Box<dyn Future<Item = SetLedErrorResponse, Error = ApiError> + Send> {
        info!(
            "set_led_error({}, {}, {}, {:?})",
            bus_id,
            addr,
            led,
            error,
        );
        warn!("No such method");
        Box::new(futures::failed("Generic failure".into()))
    }

    fn set_led_info(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        led_info: models::LedInfo,
        _context: &C,
    ) -> Box<dyn Future<Item = SetLedInfoResponse, Error = ApiError> + Send> {
        info!(
            "set_led_info({}, {}, {}, {:?})",
            bus_id,
            addr,
            led,
            led_info,
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn set_led_info_all(
        &self,
        bus_id: i32,
        addr: i32,
        led_info: models::LedInfoArray,
        _context: &C,
    ) -> Box<dyn Future<Item = SetLedInfoAllResponse, Error = ApiError> + Send> {
        info!(
            "set_led_info_all({}, {}, {:?})",
            bus_id,
            addr,
            led_info,
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn set_led_pwm(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        pwm: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetLedPwmResponse, Error = ApiError> + Send> {
        http::set_led_pwm(bus_id.into(), addr.into(), led.into(), pwm.into())
    }

    fn set_led_state(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        state: models::LedState,
        _context: &C,
    ) -> Box<dyn Future<Item = SetLedStateResponse, Error = ApiError> + Send> {
        http::set_led_state(bus_id.into(), addr.into(), led.into(), state.into())
    }

    fn set_offset(
        &self,
        bus_id: i32,
        addr: i32,
        offset: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetOffsetResponse, Error = ApiError> + Send> {
        http::set_offset(bus_id.into(), addr.into(), offset.into())
    }

    fn set_output_change(
        &self,
        bus_id: i32,
        addr: i32,
        output_change: models::OutputChange,
        _context: &C,
    ) -> Box<dyn Future<Item = SetOutputChangeResponse, Error = ApiError> + Send> {
        http::set_output_change(bus_id.into(), addr.into(), output_change.into())
    }

    fn set_pwm(
        &self,
        bus_id: i32,
        addr: i32,
        pwm: i32,
        _context: &C,
    ) -> Box<dyn Future<Item = SetPwmResponse, Error = ApiError> + Send> {
        http::set_pwm(bus_id.into(), addr.into(), pwm.into())
    }

    fn set_sleep(
        &self,
        bus_id: i32,
        addr: i32,
        sleep: bool,
        _context: &C,
    ) -> Box<dyn Future<Item = SetSleepResponse, Error = ApiError> + Send> {
        http::set_sleep(bus_id.into(), addr.into(), sleep.into())
    }
}
