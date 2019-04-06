//! Server implementation of openapi_client.
#![allow(unused_imports)]

extern crate pca9956b_api;
extern crate swagger;

use chrono;
use futures::{self, Future};
use std::collections::HashMap;
use std::marker::PhantomData;

use swagger::{Has, XSpanIdString};
use crate::http;

use pca9956b_api::models;
use pca9956b_api::{
    Api, ApiError, ClearErrorResponse, GetAddrEnabledResponse, GetAddrInfoResponse,
    GetAddrValueResponse, GetApiResponse, GetConfigResponse, GetCurrentResponse, GetErrorResponse,
    GetErrorsResponse, GetFreqResponse, GetGroupResponse, GetLedCurrentResponse,
    GetLedErrorResponse, GetLedInfoAllResponse, GetLedInfoResponse, GetLedPwmResponse,
    GetLedStateResponse, GetOffsetResponse, GetOutputChangeResponse, GetOverTempResponse,
    GetPwmResponse, GetSleepResponse, ResetResponse, SetAddrEnabledResponse, SetAddrValueResponse,
    SetConfigResponse, SetCurrentResponse, SetFreqResponse, SetGroupResponse,
    SetLedCurrentResponse, SetLedErrorResponse, SetLedInfoAllResponse, SetLedInfoResponse,
    SetLedPwmResponse, SetLedStateResponse, SetOffsetResponse, SetOutputChangeResponse,
    SetPwmResponse, SetSleepResponse,
};

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server {
            marker: PhantomData,
        }
    }
}

impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString>,
{
    fn clear_error(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = ClearErrorResponse, Error = ApiError>> {
        http::clear_error(bus_id.into(), addr.into(), true.into())
    }

    fn get_addr_enabled(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        _context: &C,
    ) -> Box<Future<Item = GetAddrEnabledResponse, Error = ApiError>> {
        http::get_addr_enabled(bus_id.into(), addr.into(), num.into())
    }

    fn get_addr_info(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        _context: &C,
    ) -> Box<Future<Item = GetAddrInfoResponse, Error = ApiError>> {
        http::get_addr_info(bus_id.into(), addr.into(), num.into())
    }

    fn get_addr_value(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        _context: &C,
    ) -> Box<Future<Item = GetAddrValueResponse, Error = ApiError>> {
        http::get_addr_value(bus_id.into(), addr.into(), num.into())
    }

    fn get_api(&self, _context: &C) -> Box<Future<Item = GetApiResponse, Error = ApiError>> {
        http::get_api()
    }

    fn get_config(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetConfigResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = GetCurrentResponse, Error = ApiError>> {
        http::get_current(bus_id.into(), addr.into())
    }

    fn get_error(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetErrorResponse, Error = ApiError>> {
        http::get_error(bus_id.into(), addr.into())
    }

    fn get_errors(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetErrorsResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = GetFreqResponse, Error = ApiError>> {
        http::get_freq(bus_id.into(), addr.into())
    }

    fn get_group(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetGroupResponse, Error = ApiError>> {
        http::get_group(bus_id.into(), addr.into())
    }

    fn get_led_current(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<Future<Item = GetLedCurrentResponse, Error = ApiError>> {
        http::get_led_current(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_error(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<Future<Item = GetLedErrorResponse, Error = ApiError>> {
        http::get_led_error(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_info(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<Future<Item = GetLedInfoResponse, Error = ApiError>> {
        http::get_led_info(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_info_all(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetLedInfoAllResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = GetLedPwmResponse, Error = ApiError>> {
        http::get_led_pwm(bus_id.into(), addr.into(), led.into())
    }

    fn get_led_state(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        _context: &C,
    ) -> Box<Future<Item = GetLedStateResponse, Error = ApiError>> {
        http::get_led_state(bus_id.into(), addr.into(), led.into())
    }

    fn get_offset(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetOffsetResponse, Error = ApiError>> {

        http::get_offset(bus_id.into(), addr.into())
    }

    fn get_output_change(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetOutputChangeResponse, Error = ApiError>> {
        http::get_output_change(bus_id.into(), addr.into())
    }

    fn get_over_temp(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetOverTempResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = GetPwmResponse, Error = ApiError>> {
        http::get_pwm(bus_id.into(), addr.into())
    }

    fn get_sleep(
        &self,
        bus_id: i32,
        addr: i32,
        _context: &C,
    ) -> Box<Future<Item = GetSleepResponse, Error = ApiError>> {
        http::get_sleep(bus_id.into(), addr.into())
    }

    fn reset(
        &self,
        bus_id: i32,
        _context: &C,
    ) -> Box<Future<Item = ResetResponse, Error = ApiError>> {
        http::reset(bus_id.into())
    }

    fn set_addr_enabled(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        enabled: bool,
        _context: &C,
    ) -> Box<Future<Item = SetAddrEnabledResponse, Error = ApiError>> {
        http::set_addr_enabled(bus_id.into(), addr.into(), num.into(), enabled.into())
    }

    fn set_addr_value(
        &self,
        bus_id: i32,
        addr: i32,
        num: i32,
        addr_val: i32,
        _context: &C,
    ) -> Box<Future<Item = SetAddrValueResponse, Error = ApiError>> {
        http::set_addr_value(bus_id.into(), addr.into(), num.into(), addr_val.into())
    }

    fn set_config(
        &self,
        bus_id: i32,
        addr: i32,
        config: models::Config,
        _context: &C,
    ) -> Box<Future<Item = SetConfigResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = SetCurrentResponse, Error = ApiError>> {
        http::set_current(bus_id.into(), addr.into(), current.into())
    }

    fn set_freq(
        &self,
        bus_id: i32,
        addr: i32,
        freq: i32,
        _context: &C,
    ) -> Box<Future<Item = SetFreqResponse, Error = ApiError>> {
        http::set_freq(bus_id.into(), addr.into(), freq.into())
    }

    fn set_group(
        &self,
        bus_id: i32,
        addr: i32,
        group: models::Group,
        _context: &C,
    ) -> Box<Future<Item = SetGroupResponse, Error = ApiError>> {
        http::set_group(bus_id.into(), addr.into(), group.into())
    }

    fn set_led_current(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        current: i32,
        _context: &C,
    ) -> Box<Future<Item = SetLedCurrentResponse, Error = ApiError>> {
        http::set_led_current(bus_id.into(), addr.into(), led.into(), current.into())
    }

    fn set_led_error(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        error: models::LedError,
        _context: &C,
    ) -> Box<Future<Item = SetLedErrorResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = SetLedInfoResponse, Error = ApiError>> {
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
        led_info: &Vec<models::LedInfo>,
        _context: &C,
    ) -> Box<Future<Item = SetLedInfoAllResponse, Error = ApiError>> {
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
    ) -> Box<Future<Item = SetLedPwmResponse, Error = ApiError>> {
        http::set_led_pwm(bus_id.into(), addr.into(), led.into(), pwm.into())
    }

    fn set_led_state(
        &self,
        bus_id: i32,
        addr: i32,
        led: i32,
        state: models::LedState,
        _context: &C,
    ) -> Box<Future<Item = SetLedStateResponse, Error = ApiError>> {
        http::set_led_state(bus_id.into(), addr.into(), led.into(), state.into())
    }

    fn set_offset(
        &self,
        bus_id: i32,
        addr: i32,
        offset: i32,
        _context: &C,
    ) -> Box<Future<Item = SetOffsetResponse, Error = ApiError>> {
        http::set_offset(bus_id.into(), addr.into(), offset.into())
    }

    fn set_output_change(
        &self,
        bus_id: i32,
        addr: i32,
        output_change: models::OutputChange,
        _context: &C,
    ) -> Box<Future<Item = SetOutputChangeResponse, Error = ApiError>> {
        http::set_output_change(bus_id.into(), addr.into(), output_change.into())
    }

    fn set_pwm(
        &self,
        bus_id: i32,
        addr: i32,
        pwm: i32,
        _context: &C,
    ) -> Box<Future<Item = SetPwmResponse, Error = ApiError>> {
        http::set_pwm(bus_id.into(), addr.into(), pwm.into())
    }

    fn set_sleep(
        &self,
        bus_id: i32,
        addr: i32,
        sleep: bool,
        _context: &C,
    ) -> Box<Future<Item = SetSleepResponse, Error = ApiError>> {
        http::set_sleep(bus_id.into(), addr.into(), sleep.into())
    }
}
