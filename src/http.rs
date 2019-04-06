//! XXX To do ...
//! * Implement the rest of the API
//! * Check for unused code
//! * Change I2cHandlingError to internal error state
//! * Could put a lock around writing - so register doesn't change between read and write
//! * PWMALL can't be read (always returns 0), so need to remove read ability
//! * Same for IREFALL
//! * Actually PWMALL can be read - doesn't always return 0, but maybe remove anyway
//! * For IREFALL PWMALL (and others?) need to write reg even if currently same value (as it applies to other registers)
//! * Could use impl Future (https://tokio.rs/docs/going-deeper/returning/), would need to change Api
//! * Do some consistent and useful logging now have framework in place
//! * Get rid of set_led_error()
//! * Maybe only read led error if error bit set
//! * Need to macroize the chained functions (get_addr_info, get_led_info)
//! * Get rid of bus_id1, 2, 3, etc from chained functions - not best way

#![allow(unused_imports)]
#![allow(dead_code)]
use i2cbus_api::client::remote::{Handle, Response};
use i2cbus_api::ApiNoContext;
use i2cbus_api::Client;
use i2cbus_api::ContextWrapperExt;
use i2cbus_api::OkOrOther;
use pca9956b_api;
use pca9956b_api::models::{
    Addr, AddrEnabled, AddrIndex, AddrInfo, BadRequest, BusId, Config, Current, Error, Freq, Group,
    LedError, LedIndex, LedInfo, LedState, Offset, OpError, OutputChange, Pwm, Sleep, Yaml,
};
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
use std::{error, fmt, thread};
use httpd_util;

use futures::sync::{mpsc, oneshot};
use futures::{
    future::{self, failed, ok, Either},
    Future, stream, Stream,
};
use std::fs;
use std::env;
use std::sync::Mutex;
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};
use tokio_core::reactor;
use uuid;

const NUM_LEDS: u32 = 24;
const REG_MODE1: u8 = 0x00;
const REG_SLEEP: u8 = REG_MODE1;
const REG_ADDR_EN: u8 = REG_MODE1;
const BIT_AIF: u8 = 7;
const BITS_AI_00_3E: u8 = 0b0000_0000;
const BITS_AI_ALL: u8 = BITS_AI_00_3E; // Not actually all - missing PWMALL, IREFALL
const BITS_AI_0A_21: u8 = 0b0010_0000;
const BITS_AI_BRIGHT: u8 = BITS_AI_0A_21; // Individual brightness (PWM) registers only
const BITS_AI_00_39: u8 = 0b0100_0000;
const BITS_AI_ALL_LEDS: u8 = BITS_AI_00_39; // From MODE1 to IREF23
const BITS_AI_08_21: u8 = 0b0110_0000; //
const BITS_AI_BRIGHT_CTRL: u8 = BITS_AI_08_21; // GRPPWM, GRPFREQ and individual brightness registers
const BIT_SLEEP: u8 = 4;
const BIT_SUBADDR1_EN: u8 = 3;
const BIT_SUBADDR2_EN: u8 = 2;
const BIT_SUBADDR3_EN: u8 = 1;
const BIT_ALLCALL_EN: u8 = 0;
const REG_MODE2: u8 = 0x01;
const REG_OVERTEMP: u8 = REG_MODE2;
const REG_ERROR: u8 = REG_MODE2;
const REG_DMBLNK: u8 = REG_MODE2;
const REG_GROUP: u8 = REG_DMBLNK;
const REG_CLRERR: u8 = REG_MODE2;
const REG_OCH: u8 = REG_MODE2;
const BIT_OVERTEMP: u8 = 7;
const BIT_ERROR: u8 = 6;
const BIT_DMBLNK: u8 = 5;
const BIT_GROUP: u8 = BIT_DMBLNK; // synonym for DMBLNK
const BIT_CLRERR: u8 = 4;
const BIT_OCH: u8 = 3;
const BITS_MODE2_RESERVED: u8 = 0b101;
const REG_LEDOUT0: u8 = 0x02;
const REG_LEDOUT1: u8 = REG_LEDOUT0 + 1;
const REG_LEDOUT2: u8 = REG_LEDOUT0 + 2;
const REG_LEDOUT3: u8 = REG_LEDOUT0 + 3;
const REG_LEDOUT4: u8 = REG_LEDOUT0 + 4;
const REG_LEDOUT5: u8 = REG_LEDOUT0 + 5;
const REG_GRPPWM: u8 = 0x08;
const_assert!(assert1; REG_GRPPWM == REG_LEDOUT5 + 1);
const REG_GRPFREQ: u8 = 0x09;
const REG_PWM0: u8 = 0x0A;
const REG_PWM1: u8 = REG_PWM0 + 1;
const REG_PWM2: u8 = REG_PWM0 + 2;
const REG_PWM3: u8 = REG_PWM0 + 3;
const REG_PWM4: u8 = REG_PWM0 + 4;
const REG_PWM5: u8 = REG_PWM0 + 5;
const REG_PWM6: u8 = REG_PWM0 + 6;
const REG_PWM7: u8 = REG_PWM0 + 7;
const REG_PWM8: u8 = REG_PWM0 + 8;
const REG_PWM9: u8 = REG_PWM0 + 9;
const REG_PWM10: u8 = REG_PWM0 + 10;
const REG_PWM11: u8 = REG_PWM0 + 11;
const REG_PWM12: u8 = REG_PWM0 + 12;
const REG_PWM13: u8 = REG_PWM0 + 13;
const REG_PWM14: u8 = REG_PWM0 + 14;
const REG_PWM15: u8 = REG_PWM0 + 15;
const REG_PWM16: u8 = REG_PWM0 + 16;
const REG_PWM17: u8 = REG_PWM0 + 17;
const REG_PWM18: u8 = REG_PWM0 + 18;
const REG_PWM19: u8 = REG_PWM0 + 19;
const REG_PWM20: u8 = REG_PWM0 + 20;
const REG_PWM21: u8 = REG_PWM0 + 21;
const REG_PWM22: u8 = REG_PWM0 + 22;
const REG_PWM23: u8 = REG_PWM0 + 23;
const REG_IREF0: u8 = 0x22;
const_assert!(assert2; REG_IREF0 == REG_PWM23 + 1);
const REG_IREF1: u8 = REG_IREF0 + 1;
const REG_IREF2: u8 = REG_IREF0 + 2;
const REG_IREF3: u8 = REG_IREF0 + 3;
const REG_IREF4: u8 = REG_IREF0 + 4;
const REG_IREF5: u8 = REG_IREF0 + 5;
const REG_IREF6: u8 = REG_IREF0 + 6;
const REG_IREF7: u8 = REG_IREF0 + 7;
const REG_IREF8: u8 = REG_IREF0 + 8;
const REG_IREF9: u8 = REG_IREF0 + 9;
const REG_IREF10: u8 = REG_IREF0 + 10;
const REG_IREF11: u8 = REG_IREF0 + 11;
const REG_IREF12: u8 = REG_IREF0 + 12;
const REG_IREF13: u8 = REG_IREF0 + 13;
const REG_IREF14: u8 = REG_IREF0 + 14;
const REG_IREF15: u8 = REG_IREF0 + 15;
const REG_IREF16: u8 = REG_IREF0 + 16;
const REG_IREF17: u8 = REG_IREF0 + 17;
const REG_IREF18: u8 = REG_IREF0 + 18;
const REG_IREF19: u8 = REG_IREF0 + 19;
const REG_IREF20: u8 = REG_IREF0 + 20;
const REG_IREF21: u8 = REG_IREF0 + 21;
const REG_IREF22: u8 = REG_IREF0 + 22;
const REG_IREF23: u8 = REG_IREF0 + 23;
const REG_OFFSET: u8 = 0x3A;
const_assert!(assert3; REG_OFFSET == REG_IREF23 + 1);
const REG_SUBADR1: u8 = 0x3B;
const REG_SUBADR2: u8 = 0x3C;
const REG_SUBADR3: u8 = 0x3D;
const REG_ALLCALLADR: u8 = 0x3E;
const REG_PWMALL: u8 = 0x3F;
const REG_IREFALL: u8 = 0x40;
const REG_EFLAG0: u8 = 0x41;
const REG_EFLAG1: u8 = REG_EFLAG0 + 1;
const REG_EFLAG2: u8 = REG_EFLAG0 + 2;
const REG_EFLAG3: u8 = REG_EFLAG0 + 3;
const REG_EFLAG4: u8 = REG_EFLAG0 + 4;
const REG_EFLAG5: u8 = REG_EFLAG0 + 5;
const_assert!(assert4; REG_EFLAG5 == 0x46);

const I2CBUS_IP_VAR: &str = "I2CBUS_IP";
const I2CBUS_IP_DEF: &str = "0.0.0.0";
const I2CBUS_PORT_VAR: &str = "I2CBUS_PORT";
const I2CBUS_PORT_DEF: &str = "8080";
const I2CBUS_HTTPS_VAR: &str = "I2CBUS_HTTPS";

pub fn get_env() -> Vec<&'static str> {
    vec![I2CBUS_IP_VAR, I2CBUS_PORT_VAR, I2CBUS_HTTPS_VAR]
}

lazy_static! {
    static ref BASE_URL: String = {
        let addr = httpd_util::get_addr(I2CBUS_IP_VAR, I2CBUS_IP_DEF, I2CBUS_PORT_VAR, I2CBUS_PORT_DEF);
        let proto = match env::var(I2CBUS_HTTPS_VAR) {
            Ok(_) => "https",
            Err(_) => "http",
        };
        let addr = format!("{}://{}", proto, addr);
        addr
    };
}

lazy_static! {
    static ref I2CBUS_HANDLE: Handle = Handle::new(&BASE_URL);
}

/// Define our own error type.
///
/// We have to handle error responses from i2cbus-api, which sometimes look
/// like errors (ApiError) and sometimes look like successes (e.g.
/// I2cBusWriteBytesResponse::TransactionFailed).
///
/// However, we want to map i2cbus-api errors which will prevent us from
/// continuing with executing the desired operation to an actual error type
/// as this makes future handling much easier.
///
/// So, we will map all i2cbus-api errors to the single ProcError (Processing
/// Error) type, while retaining as much information as possible on the
/// original failure.
///
/// We don't need to store any info about OK responses (that do include the
/// appropriate Some(_) detail), because they are actually OKs!
///
/// XXX Would be good to include API call information in here (e.g. which I2C bus API was called)
///
#[derive(Debug)]
enum I2cHandlingError {
    // When Api returns OK but mandatory extra info isn't provided
    NoOkInfo(ErroredApi),

    // Returned by /i2c/api only
    FileNotFound(ErroredApi, i2cbus_api::models::Error),

    // Usually indicates some error with the info passed in on the API call
    BadRequest(ErroredApi, i2cbus_api::models::I2cBusArg),

    // Indicates some sort of failure on the I2C bus - which could be a physical problem or undeteted issue with input args
    TransactionFailed(ErroredApi, i2cbus_api::models::I2cBusError),

    // Usually indicates some sort of failure of the actual i2cbus-api client implementation
    ApiError(ErroredApi, ApiError),

    // An error with this module
    PcaError(String),
}

#[allow(dead_code)]
#[derive(Debug)]
enum ErroredApi {
    I2cBusApiResponse,
    I2cBusListResponse,
    I2cBusReadByteResponse,
    I2cBusReadBytesResponse,
    I2cBusReadRegResponse,
    I2cBusWriteByteResponse,
    I2cBusWriteByteRegResponse,
    I2cBusWriteBytesResponse,
    I2cBusWriteBytesRegResponse,
    Unknown,
}

impl fmt::Display for I2cHandlingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            I2cHandlingError::NoOkInfo(ref api) => write!(f, "No OK Info from I2C API: {}", api),
            I2cHandlingError::FileNotFound(ref api, ref err) => write!(
                f,
                "FileNotFound error from I2C API: {}, error info: {:?}",
                api, err
            ),
            I2cHandlingError::BadRequest(ref api, ref err) => write!(
                f,
                "BadRequest error from I2C API: {}, error info: {:?}",
                api, err
            ),
            I2cHandlingError::TransactionFailed(ref api, ref err) => write!(
                f,
                "TransactionFailed from I2C API: {}, error info: {:?}",
                api, err
            ),
            I2cHandlingError::ApiError(ref api, ref err) => {
                write!(f, "ApiError from I2C API: {}, error info: {:?}", api, err)
            }
            I2cHandlingError::PcaError(ref err) => write!(f, "PcaError: {}", err),
        }
    }
}

impl fmt::Display for ErroredApi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErroredApi::I2cBusApiResponse => write!(f, "I2cBusApiResponse"),
            ErroredApi::I2cBusListResponse => write!(f, "I2cBusListResponse"),
            ErroredApi::I2cBusReadByteResponse => write!(f, "I2cBusReadByteResponse"),
            ErroredApi::I2cBusReadBytesResponse => write!(f, "I2cBusReadByteResponse"),
            ErroredApi::I2cBusReadRegResponse => write!(f, "I2cBusReadRegResponse"),
            ErroredApi::I2cBusWriteByteResponse => write!(f, "I2cBusWriteByteResponse"),
            ErroredApi::I2cBusWriteByteRegResponse => write!(f, "I2cBusWriteByteRegResponse"),
            ErroredApi::I2cBusWriteBytesResponse => write!(f, "I2cBusWriteBytesResponse"),
            ErroredApi::I2cBusWriteBytesRegResponse => write!(f, "I2cBusWriteBytesRegResponse"),
            ErroredApi::Unknown => write!(f, "Unknown"),
        }
    }
}

impl error::Error for I2cHandlingError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            I2cHandlingError::NoOkInfo(_) => None,
            I2cHandlingError::FileNotFound(_, ref _err) => None, // As err here doesn't actually implement an error::Error trait
            I2cHandlingError::BadRequest(_, ref _err) => None, // As err here doesn't actually implement an error::Error trait
            I2cHandlingError::TransactionFailed(_, ref _err) => None, // As err here doesn't actually implement an error::Error trait
            I2cHandlingError::ApiError(_, ref err) => Some(err),
            I2cHandlingError::PcaError(_) => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct BusList {
    path: String,
    id: i32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct BusRead {
    ok: i32,
    values: Vec<i32>,
}

#[derive(Debug)]
enum OkData {
    I2cBusYaml(String),
    I2cBusList(Vec<BusList>),
    I2cBusRead(BusRead),
    I2cBusOk(i32),
}

macro_rules! return_none {
    () => {
        return Box::new(failed(I2cHandlingError::NoOkInfo(ErroredApi::Unknown)));
    };
}

macro_rules! unwrap_or_return_if_none {
    ($($x:tt)*) => {
        match $($x)* {
            Some(val) => val,
            None => return_none!(),
        }
    }
}

macro_rules! map_i2c_handling_err_to_rsp {
    ($rsp_ty: tt, $err: tt) => {
        match $err {
            I2cHandlingError::BadRequest(_a, x) => $rsp_ty::BadRequest(BadRequest {
                parameter: x.arg,
                error: x.description,
            }),
            I2cHandlingError::TransactionFailed(_a, x) => $rsp_ty::OperationFailed(OpError {
                error: Some(
                    format!(
                        "I2C bus error: {} {}",
                        x.error.unwrap_or(0),
                        x.description.unwrap_or("".to_string())
                    )
                    .to_string(),
                ),
            }),
            I2cHandlingError::NoOkInfo(a) => $rsp_ty::OperationFailed(OpError {
                error: Some(format!("I2C bus error: NoOkInfo {}", a)),
            }),
            I2cHandlingError::PcaError(e) => $rsp_ty::OperationFailed(OpError {
                error: Some(format!("PCA error: {}", e)),
            }),
            I2cHandlingError::FileNotFound(a, e) => $rsp_ty::OperationFailed(OpError {
                error: Some(format!(
                    "I2C bus error: FileNotFound {} {}",
                    a,
                    e.to_string()
                )),
            }),
            I2cHandlingError::ApiError(a, e) => $rsp_ty::OperationFailed(OpError {
                error: Some(format!("I2C bus error: ApiError {} {}", a, e)),
            }),
        }
    };
}

macro_rules! unexpected_i2c_rsp {
    ($rsp_ty: tt, $info: tt) => {
        $rsp_ty::OperationFailed(OpError {
            error: Some(format!("Unexpected I2C Bus Response: {:?}", $info)),
        })
    };
}

macro_rules! handle_i2cbus_ok_err {
    ($rsp: tt, $ok_ty:tt, $rsp_ty: tt, $ok_fn: tt) => {
        Box::new(ok(match $rsp {
            Ok(info) => match info {
                OkData::$ok_ty(x) => $ok_fn(x),
                _ => unexpected_i2c_rsp!($rsp_ty, info),
            },
            Err(e) => map_i2c_handling_err_to_rsp!($rsp_ty, e),
        }))
    };
}

macro_rules! convert_int_error_to_api_error {
    ($x: tt, $rsp_ty: tt) => {
        match $x {
            Ok(x) => Ok(x),
            Err(e) => Ok(map_i2c_handling_err_to_rsp!($rsp_ty, e)),
        }
    };
}

macro_rules! int_handle_i2cbus_ok_err {
    ($rsp: tt, $ok_ty:tt, $ok_fn: tt) => {
        Box::new(match $rsp {
            Ok(info) => match info {
                OkData::$ok_ty(x) => ok($ok_fn(x)),
                _ => failed(I2cHandlingError::TransactionFailed(
                    ErroredApi::I2cBusReadRegResponse,
                    i2cbus_api::models::I2cBusError {
                        error: Some(0),
                        description: Some(
                            format!("Unexpected I2C Bus Response: {:?}", info).to_string(),
                        ),
                    },
                )),
            },
            Err(e) => failed(e),
        })
    };
}

macro_rules! handle_i2cbus_response {
    ($rsp: tt, $ok_ty:tt, $ok_fn: tt) => {
        i2cbus_response_to_ok_err($rsp).then(|x| int_handle_i2cbus_ok_err!(x, $ok_ty, $ok_fn))
    };
}

fn i2cbus_response_to_ok_err<T>(
    i2c_rsp: Result<T, ApiError>,
) -> Box<Future<Item = OkData, Error = I2cHandlingError>>
where
    T: i2cbus_api::OkOrOther,
{
    match i2c_rsp {
        Ok(x) => match x.ok_or_other() {
            Ok(x) => Box::new(ok(match x {
                // Just convert the Yaml data
                i2cbus_api::ExtraInfoOk::Yaml(x) => {
                    debug!("I2C Bus Response: Yaml {:?}", x);
                    OkData::I2cBusYaml(x.into())
                }

                // Convert the bus list - note an empty bus list is OK
                i2cbus_api::ExtraInfoOk::List(x) => {
                    debug!("I2C Bus Response: List {:?}", x);
                    OkData::I2cBusList({
                        let mut bus_list = Vec::with_capacity(x.len());
                        for x in x {
                            let path = unwrap_or_return_if_none!(x.path);
                            let id = unwrap_or_return_if_none!(x.id);
                            bus_list.push(BusList { path, id });
                        }
                        bus_list
                    })
                }

                // Convert the read data - empty data is _not_ OK
                i2cbus_api::ExtraInfoOk::Read(x) => {
                    debug!("I2C Bus Response: Read {:?}", x);
                    OkData::I2cBusRead(BusRead {
                        ok: unwrap_or_return_if_none!(x.ok),
                        values: {
                            let values = unwrap_or_return_if_none!(x.values);
                            if values.len() == 0 {
                                return_none!();
                            } else {
                                values.iter().map(|x| i32::from(x.clone())).collect()
                            }
                        },
                    })
                }

                // Convert the OK response code
                i2cbus_api::ExtraInfoOk::OK(x) => {
                    debug!("I2C Bus Response: OK {:?}", x);
                    OkData::I2cBusOk(unwrap_or_return_if_none!(x.ok))
                }
            })),
            Err(e) => Box::new(failed(match e {
                i2cbus_api::ExtraInfoError::FileNotFound(x) => {
                    debug!("I2C Bus Error Response: FileNotFound {:?}", x);
                    I2cHandlingError::FileNotFound(ErroredApi::Unknown, x)
                } // XXX Todo implement ErroredApi
                i2cbus_api::ExtraInfoError::Arg(x) => {
                    debug!("I2C Bus Error Response: Arg {:?}", x);
                    I2cHandlingError::BadRequest(ErroredApi::Unknown, x)
                }
                i2cbus_api::ExtraInfoError::Error(x) => {
                    debug!("I2C Bus Error Response: Error {:?}", x);
                    I2cHandlingError::TransactionFailed(ErroredApi::Unknown, x)
                }
            })),
        },
        Err(e) => {
            debug!("I2C Bus Error Response: ApiError {:?}", e);
            Box::new(failed(I2cHandlingError::ApiError(ErroredApi::Unknown, e)))
        }
    }
}

fn get_reg_val(
    bus_id: &BusId,
    addr: &Addr,
    reg: &u8,
) -> Box<Future<Item = u8, Error = I2cHandlingError>> {
    Box::new(
        I2CBUS_HANDLE
            .i2c_bus_read_reg(
                // Read the register containing the current sleep value
                i32::from(bus_id.clone()).into(),
                i32::from(addr.clone()).into(),
                i32::from(reg.clone()).into(),
                1.into(), // Read 1 byte
            )
            .then(|x| handle_i2cbus_response!(x, I2cBusRead, get_reg_val_handle_rsp)),
    )
}

fn get_reg_val_handle_rsp(rsp: BusRead) -> u8 {
    let reg_val: i32 = rsp.values[0].into();
    debug!("Read register value 0b{:0>8b}", reg_val);
    reg_val as u8
}

fn set_reg_val(
    bus_id: &BusId,
    addr: &Addr,
    reg: &u8,
    val: &u8,
) -> Box<Future<Item = (), Error = I2cHandlingError>> {
    Box::new(
        I2CBUS_HANDLE
            .i2c_bus_write_byte_reg(
                i32::from(bus_id.clone()).into(),
                i32::from(addr.clone()).into(),
                i32::from(reg.clone()).into(),
                i32::from(val.clone()).into(),
            )
            .then(|x| handle_i2cbus_response!(x, I2cBusOk, set_reg_val_handle_rsp)),
    )
}

fn set_reg_val_handle_rsp(_rsp: i32) -> () {
    ()
}

/// Macro called to check whether a value is within bounds
macro_rules! bounds_check {
    ($extra:tt, $extra_str:tt, $extra_test_fn:tt, $rsp_ty:tt, $success_do:expr) => {{
        if !$extra_test_fn(&$extra) {
            debug!(
                "Requested {} value out of bounds {:?}",
                $extra_str,
                $extra.clone()
            );
            Either::A(ok($rsp_ty::BadRequest(BadRequest {
                parameter: Some($extra_str.to_string()),
                error: Some("Value out of range".to_string()),
            })))
        } else {
            Either::B($success_do)
        }
    }};
}

/// Macro used to get a bit of a register
macro_rules! make_bit_get {
    ($fn:tt, $input_ty:tt, $rsp_ty:tt, $reg_c:tt, $bit_c:tt, $fn_get_val:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?}", stringify!($fn), bus_id, addr);
            Box::new(
                get_reg_val(&bus_id, &addr, &$reg_c)
                    .and_then(|val| ok($input_ty::from($fn_get_val(&val, &$bit_c))))
                    .and_then(|x| ok($rsp_ty::OK(x.into())))
                    .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
    // Used when another variable is input - e.g. AddrIndex
    ($fn:tt, $input_ty:tt, $extra:tt, $extra_ty:tt, $rsp_ty:tt, $reg_c:tt, $bit_c:tt, $fn_get_val:tt, $extra_test_fn:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
            extra: $extra_ty,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?} {:?}", stringify!($fn), bus_id, addr, extra);
            Box::new(
                bounds_check!(extra, $extra, $extra_test_fn, $rsp_ty, {
                    get_reg_val(&bus_id, &addr, &$reg_c)
                        .and_then(move |val| ok($input_ty::from($fn_get_val(&val, &$bit_c, &extra))))
                        .and_then(|x| ok($rsp_ty::OK(x.into())))
                        .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                })
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
}

/// Macro used to set a bit of a register
macro_rules! make_bit_set {
    ($fn:tt, $input_ty:tt, $rsp_ty:tt, $reg_c:tt, $bit_c:tt, $fn_get_val:tt, $fn_set_reg_val:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
            input_var: $input_ty,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?} {:?}", stringify!($fn), bus_id, addr, input_var);
            Box::new(
                get_reg_val(&bus_id, &addr, &$reg_c)
                    .and_then(move |reg| {
                        ok($input_ty::from($fn_get_val(&reg, &$bit_c))).and_then(move |_old_val| {
                            let new_val = $fn_set_reg_val(&input_var.into(), &reg, &$bit_c);
                            set_reg_val(&bus_id, &addr, &$reg_c, &new_val)
                                .and_then(|_| ok($rsp_ty::OK))
                        })
                    })
                    .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
    // Used when another variable is input - e.g. AddrIndex
    ($fn:tt, $input_ty:tt, $extra:tt, $extra_ty:tt, $rsp_ty:tt, $reg_c:tt, $bit_c:tt, $fn_get_val:tt, $fn_set_reg_val:tt, $extra_test_fn:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
            extra: $extra_ty,
            input_var: $input_ty,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?} {:?} {:?}", stringify!($fn), bus_id, addr, extra, input_var);
            Box::new(
                bounds_check!(extra, $extra, $extra_test_fn, $rsp_ty, {
                    get_reg_val(&bus_id, &addr, &$reg_c)
                        .and_then(move |reg| {
                            ok($input_ty::from($fn_get_val(&reg, &$bit_c, &extra))).and_then(
                                move |_old_val| {
                                    let new_val = $fn_set_reg_val(
                                        &input_var.into(),
                                        &reg,
                                        &$bit_c,
                                        &extra,
                                    );
                                    set_reg_val(&bus_id, &addr, &$reg_c, &new_val)
                                        .and_then(|_| ok($rsp_ty::OK))
                                },
                            )
                        })
                        .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                })
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
}

/// Macro used to get a whole register (a complete byte)
macro_rules! make_reg_get {
    ($fn:tt, $rsp_ty:tt, $reg_c:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?}", stringify!($fn), bus_id, addr);
            Box::new(
                get_reg_val(&bus_id, &addr, &$reg_c)
                    .and_then(|val| ok($rsp_ty::OK(val.into())))
                    .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
    // Used when an extra value is used to identify the register
    ($fn:tt, $extra:tt, $extra_ty:tt, $rsp_ty:tt, $reg_c:tt, $fn_get_reg:tt, $fn_get_val:tt, $extra_test_fn:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
            extra: $extra_ty,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?} {:?}", stringify!($fn), bus_id, addr, extra);
            Box::new(
                bounds_check!(extra, $extra, $extra_test_fn, $rsp_ty, {
                    get_reg_val(&bus_id, &addr, &$fn_get_reg(&$reg_c, &extra))
                        .and_then(move |val| ok($rsp_ty::OK($fn_get_val(&val, &extra).into())))
                        .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                })
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
}

/// Macro used to set a whole register (a complete byte)
macro_rules! make_reg_set {
    ($fn:tt, $input_var:tt, $input_ty:tt, $rsp_ty:tt, $reg:tt, $reg_c:tt, $test_fn:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
            input_var: $input_ty,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?} {:?}", stringify!($fn), bus_id, addr, input_var);
            Box::new({
                bounds_check!(input_var, $input_var, $test_fn, $rsp_ty, {
                    get_reg_val(&bus_id, &addr, &$reg_c)
                        .and_then(move |_val| {
                            let input_var = i32::from(input_var) as u8;
                            debug!("Writing to {} register value 0b{:0>8b}", $reg, input_var);
                            set_reg_val(&bus_id, &addr, &$reg_c, &input_var)
                                .and_then(|_| ok($rsp_ty::OK))
                        })
                        .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                })
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            })
        }
    };
    // Used when an extra value is used to identify the register
    ($fn:tt, $input_var:tt, $input_ty:tt, $extra:tt, $extra_ty:tt, $rsp_ty:tt, $reg:tt, $reg_c:tt, $fn_get_reg:tt, $fn_get_val:tt, $input_test_fn:tt, $extra_test_fn:tt) => {
        pub(crate) fn $fn(
            bus_id: BusId,
            addr: Addr,
            extra: $extra_ty,
            input_var: $input_ty,
        ) -> Box<Future<Item = $rsp_ty, Error = ApiError>> {
            info!("API {} : {:?} {:?} {:?} {:?}", stringify!($fn), bus_id, addr, extra, input_var);
            Box::new(
                bounds_check!(extra, $extra, $extra_test_fn, $rsp_ty, {
                    bounds_check!(input_var, $input_var, $input_test_fn, $rsp_ty, {
                        let reg = $fn_get_reg(&$reg_c, &extra);
                        get_reg_val(&bus_id, &addr, &reg)
                            .and_then(move |val| {
                                let new_val = $fn_get_val(&input_var, &extra, &val);
                                set_reg_val(&bus_id, &addr, &reg, &new_val)
                                    .and_then(|_| ok($rsp_ty::OK))
                            })
                            .then(|x| convert_int_error_to_api_error!(x, $rsp_ty))
                    })
                })
                    .then(|x| { info!("API {} -> {:?}", stringify!($fn), x); x } )
            )
        }
    };
}

/// This method gets a boolean from a bit of a register
fn get_bool_val_from_reg(reg: &u8, bit: &u8) -> bool {
    (reg.clone() & ((1 << bit) as u8)) != 0
}

/// This method sets a register based on a boolean bit value
fn set_reg_from_bool_val(val: &bool, old_reg: &u8, bit: &u8) -> u8 {
    if val.clone() {
        let mask = (1 as u8) << bit;
        old_reg | mask
    } else {
        let mask = !((1 as u8) << bit);
        old_reg & mask
    }
}

fn get_group_val_from_reg(reg: &u8, bit: &u8) -> Group {
    if (reg.clone() & ((1 as u8) << bit)) != 0 {
        Group::BLINK
    } else {
        Group::DIM
    }
}

fn set_reg_from_group_val(val: &Group, old_reg: &u8, bit: &u8) -> u8 {
    let new_reg_val = old_reg & (!((1 as u8) << bit));
    let val = match val {
        Group::DIM => false,
        Group::BLINK => true,
    };
    new_reg_val | (u8::from(val) << bit)
}

fn get_output_change_val_from_reg(reg: &u8, bit: &u8) -> OutputChange {
    if (reg.clone() & ((1 as u8) << bit)) != 0 {
        OutputChange::ACK
    } else {
        OutputChange::STOP
    }
}

fn set_reg_from_output_change_val(val: &OutputChange, old_reg: &u8, bit: &u8) -> u8 {
    let new_reg_val = old_reg & (!((1 as u8) << bit));
    let val = match val {
        OutputChange::STOP => false,
        OutputChange::ACK => true,
    };
    new_reg_val | (u8::from(val) << bit)
}

fn get_error_val_from_reg(reg: &u8, bit: &u8) -> Error {
    if (reg.clone() & ((1 as u8) << bit)) != 0 {
        true.into()
    } else {
        false.into()
    }
}

fn set_reg_from_error_val(val: &Error, old_reg: &u8, bit: &u8) -> u8 {
    let new_reg_val = old_reg & (!((1 as u8) << bit));
    let val = match bool::from(val.clone()) {
        true => true,
        false => {
            assert!(false, "Can't set CLRERR to 0");
            false
        },
    };
    new_reg_val | (u8::from(val) << bit)
}

pub(crate) fn get_api() -> Box<Future<Item = GetApiResponse, Error = ApiError>> {
    // Read in the file
    info!("API get_api");
    let rsp = match fs::read("/static/api.yaml") {
        Ok(api) => match String::from_utf8(api) {
            Ok(s) => GetApiResponse::OK(s.to_string()),
            Err(e) => GetApiResponse::FileNotFound(
                format!("Hit error parsing API file {}", e).to_string(),
            ),
        },
        Err(e) => GetApiResponse::FileNotFound(format!("{}", e).to_string()),
    };
    match rsp {
        GetApiResponse::OK(_) => info!("API get_api -> OK(api)"),  // Don't log entire api!
        GetApiResponse::FileNotFound(_) => info!("API get_api -> {:?}", rsp),
    };
    Box::new(ok(rsp))
}

pub(crate) fn reset(bus_id: BusId) -> Box<Future<Item = ResetResponse, Error = ApiError>> {
    let bus_id: i32 = bus_id.into();
    info!("API reset : {:?}", bus_id);
    Box::new({
        info!("Issuing SWRST on bus {}", i32::from(bus_id.clone()));
        I2CBUS_HANDLE
            .i2c_bus_write_bytes(
                bus_id.into(),
                0.into(), // Use 0 address for SWRST
                i2cbus_api::models::Values {
                    values: Some(vec![6.into()]), // Value/register 6 to trigger SWRST
                },
            )
            .then(
                // Convert write_bytes response into a Result<OkData, I2cHandlingError>
                |x| i2cbus_response_to_ok_err(x),
            )
            .then(
                // Convert to final response types
                |x| handle_i2cbus_ok_err!(x, I2cBusOk, ResetResponse, reset_i2cbus_ok),
            )
            .then(|x| { info!("API reset -> {:?}", x); x } )
    })
}

fn reset_i2cbus_ok(_rsp: i32) -> ResetResponse {
    // XXX Should really test rsp is as expected (1?)
    ResetResponse::OK
}

make_bit_get!(
    get_sleep,
    Sleep,
    GetSleepResponse,
    REG_SLEEP,
    BIT_SLEEP,
    get_bool_val_from_reg
);
make_bit_set!(
    set_sleep,
    Sleep,
    SetSleepResponse,
    REG_SLEEP,
    BIT_SLEEP,
    get_bool_val_from_reg,
    set_reg_from_bool_val
);

make_bit_get!(
    get_group,
    Group,
    GetGroupResponse,
    REG_GROUP,
    BIT_GROUP,
    get_group_val_from_reg
);
make_bit_set!(
    set_group,
    Group,
    SetGroupResponse,
    REG_GROUP,
    BIT_GROUP,
    get_group_val_from_reg,
    set_reg_from_group_val
);

make_bit_get!(
    get_output_change,
    OutputChange,
    GetOutputChangeResponse,
    REG_OCH,
    BIT_OCH,
    get_output_change_val_from_reg
);
make_bit_set!(
    set_output_change,
    OutputChange,
    SetOutputChangeResponse,
    REG_OCH,
    BIT_OCH,
    get_output_change_val_from_reg,
    set_reg_from_output_change_val
);

make_bit_get!(
    get_error,
    Error,
    GetErrorResponse,
    REG_ERROR,
    BIT_ERROR,
    get_error_val_from_reg
);
make_bit_set!(
    clear_error,
    Error,
    ClearErrorResponse,
    REG_CLRERR,
    BIT_CLRERR,
    get_error_val_from_reg,
    set_reg_from_error_val
);

make_bit_get!(
    get_addr_enabled,
    AddrEnabled,
    "num",
    AddrIndex,
    GetAddrEnabledResponse,
    REG_ADDR_EN,
    BIT_ALLCALL_EN,
    get_bool_val_from_reg_addr_index,
    bounds_check_addr_index
);
make_bit_set!(
    set_addr_enabled,
    AddrEnabled,
    "num",
    AddrIndex,
    SetAddrEnabledResponse,
    REG_ADDR_EN,
    BIT_ALLCALL_EN,
    get_bool_val_from_reg_addr_index,
    set_reg_from_bool_val_addr_index,
    bounds_check_addr_index
);
fn bounds_check_addr_index(val: &AddrIndex) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 3
}
fn convert_addr_index_to_bit(val: &AddrIndex) -> u8 {
    let index = i32::from(val.clone()) as u8;
    if index == 1 {
        3
    } else if index == 3 {
        1
    } else if (index == 0) || (index == 2) {
        index
    } else {
        assert!(
            true,
            "Invalid AddrInfo shouldn't have made it into this function"
        );
        0
    }
}
fn get_bool_val_from_reg_addr_index(reg: &u8, bit: &u8, extra: &AddrIndex) -> bool {
    let extra = convert_addr_index_to_bit(extra);
    (reg.clone() & (1 << (bit + extra) as u8)) != 0
}
fn set_reg_from_bool_val_addr_index(val: &bool, old_reg: &u8, bit: &u8, extra: &AddrIndex) -> u8 {
    let extra = convert_addr_index_to_bit(extra);
    if val.clone() {
        let mask = (1 as u8) << (bit + extra);
        old_reg | mask
    } else {
        let mask = !((1 as u8) << (bit + extra));
        old_reg & mask
    }
}

make_reg_get!(get_offset, GetOffsetResponse, REG_OFFSET);
make_reg_set!(
    set_offset,
    "offset",
    Offset,
    SetOffsetResponse,
    "OFFSET",
    REG_OFFSET,
    bounds_check_offset
);
fn bounds_check_offset(val: &Offset) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 11
}

make_reg_get!(get_pwm, GetPwmResponse, REG_GRPPWM);
make_reg_set!(
    set_pwm,
    "pwm",
    Pwm,
    SetPwmResponse,
    "GRPPWM",
    REG_GRPPWM,
    bounds_check_pwm
);
fn bounds_check_pwm(val: &Pwm) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 255
}

make_reg_get!(get_freq, GetFreqResponse, REG_GRPFREQ);
make_reg_set!(
    set_freq,
    "freq",
    Freq,
    SetFreqResponse,
    "GRPFREQ",
    REG_GRPFREQ,
    bounds_check_freq
);
fn bounds_check_freq(val: &Freq) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 255
}

make_reg_get!(get_current, GetCurrentResponse, REG_IREFALL);
make_reg_set!(
    set_current,
    "current",
    Current,
    SetCurrentResponse,
    "IREFALL",
    REG_IREFALL,
    bounds_check_current
);
fn bounds_check_current(val: &Current) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 255
}

make_reg_get!(
    get_addr_value,
    "num",
    AddrIndex,
    GetAddrValueResponse,
    REG_SUBADR1,
    get_addr_reg_from_reg,
    get_addr_val_from_reg,
    bounds_check_addr_index
);
make_reg_set!(
    set_addr_value,
    "addrVal",
    Addr,
    "num",
    AddrIndex,
    SetAddrValueResponse,
    "SUBADDR1",
    REG_SUBADR1,
    get_addr_reg_from_reg,
    set_reg_from_addr_val,
    bounds_check_addr_val,
    bounds_check_addr_index
);
fn get_addr_reg_from_reg(base_reg: &u8, val: &AddrIndex) -> u8 {
    convert_addr_index_to_addr_reg(val) + base_reg
}
fn convert_addr_index_to_addr_reg(val: &AddrIndex) -> u8 {
    let index = i32::from(val.clone()) as u8;
    match index {
        0 => REG_ALLCALLADR - REG_SUBADR1, // ALLCALLADR
        1 => REG_SUBADR1 - REG_SUBADR1,    // SUBADR1
        2 => REG_SUBADR2 - REG_SUBADR1,    // SUBADR2
        3 => REG_SUBADR3 - REG_SUBADR1,    // SUBADR3
        _ => {
            assert!(
                true,
                "Invalid AddrInfo shouldn't have made it into this function"
            );
            0
        }
    }
}
fn bounds_check_addr_val(val: &Addr) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 0x7f
}
fn get_addr_val_from_reg(reg: &u8, _index: &AddrIndex) -> Addr {
    i32::from(reg >> 1).into()
}
fn set_reg_from_addr_val(val: &Addr, _index: &AddrIndex, _old_reg: &u8) -> u8 {
    (i32::from(val.clone()) as u8) << 1
}

make_reg_get!(
    get_led_pwm,
    "led",
    LedIndex,
    GetLedPwmResponse,
    REG_PWM0,
    get_led_pwm_reg_from_reg,
    get_led_pwm_val_from_reg,
    bounds_check_led_index
);
make_reg_set!(
    set_led_pwm,
    "pwm",
    Pwm,
    "led",
    LedIndex,
    SetLedPwmResponse,
    "PWM0",
    REG_PWM0,
    get_led_pwm_reg_from_reg,
    set_reg_from_pwm_val,
    bounds_check_led_pwm,
    bounds_check_led_index
);
fn get_led_pwm_reg_from_reg(base_reg: &u8, val: &LedIndex) -> u8 {
    base_reg + (i32::from(val.clone()) as u8)
}
fn bounds_check_led_pwm(val: &Pwm) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 0xff
}
fn bounds_check_led_index(val: &LedIndex) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 23
}
fn get_led_pwm_val_from_reg(reg: &u8, _index: &LedIndex) -> Pwm {
    i32::from(*reg).into()
}
fn set_reg_from_pwm_val(val: &Pwm, _index: &LedIndex, _old_reg: &u8) -> u8 {
    i32::from(val.clone()) as u8
}

make_reg_get!(
    get_led_current,
    "led",
    LedIndex,
    GetLedCurrentResponse,
    REG_IREF0,
    get_led_current_reg_from_reg,
    get_led_current_val_from_reg,
    bounds_check_led_index
);
make_reg_set!(
    set_led_current,
    "current",
    Current,
    "led",
    LedIndex,
    SetLedCurrentResponse,
    "IREF0",
    REG_IREF0,
    get_led_current_reg_from_reg,
    set_reg_from_current_val,
    bounds_check_led_current,
    bounds_check_led_index
);
fn get_led_current_reg_from_reg(base_reg: &u8, val: &LedIndex) -> u8 {
    base_reg + (i32::from(val.clone()) as u8)
}
fn bounds_check_led_current(val: &Current) -> bool {
    let val = i32::from(val.clone());
    val >= 0 && val <= 0xff
}
fn get_led_current_val_from_reg(reg: &u8, _index: &LedIndex) -> Current {
    i32::from(*reg).into()
}
fn set_reg_from_current_val(val: &Current, _index: &LedIndex, _old_reg: &u8) -> u8 {
    i32::from(val.clone()) as u8
}

make_reg_get!(
    get_led_state,
    "led",
    LedIndex,
    GetLedStateResponse,
    REG_LEDOUT0,
    get_led_state_reg_from_reg,
    get_led_state_val_from_reg,
    bounds_check_led_index
);
make_reg_set!(
    set_led_state,
    "state",
    LedState,
    "led",
    LedIndex,
    SetLedStateResponse,
    "LEDOUT0",
    REG_LEDOUT0,
    get_led_state_reg_from_reg,
    set_reg_from_state_val,
    bounds_check_led_state,
    bounds_check_led_index
);
fn get_led_state_reg_from_reg(base_reg: &u8, val: &LedIndex) -> u8 {
    base_reg + (i32::from(val.clone()) / 4) as u8
}
fn bounds_check_led_state(_val: &LedState) -> bool {
    // No need to check as LedState is an enum
    true
}
fn get_led_state_val_from_reg(reg: &u8, index: &LedIndex) -> LedState {
    let rshift = (i32::from(index.clone()) % 4) * 2;
    let reg = reg >> rshift;
    let state: u8 = reg & 0b11;
    match state {
        0b00 => LedState::FALSE,
        0b01 => LedState::TRUE,
        0b10 => LedState::PWM,
        0b11 => LedState::PWMPLUS,
        _ => {
            assert!(false, "Somehow reg & 0b11 is greater than 3!");
            LedState::FALSE
        },
    }
}
fn set_reg_from_state_val(val: &LedState, index: &LedIndex, old_reg: &u8) -> u8 {
    // Clear out old bits
    let shift_by = (i32::from(index.clone()) % 4) * 2;
    let mask = !(0b11 << shift_by);
    let reg = old_reg & mask;

    // Set new bits
    let mask = match val {
        LedState::FALSE => 0b00,
        LedState::TRUE => 0b01,
        LedState::PWM => 0b10,
        LedState::PWMPLUS => 0b11,
    } << shift_by;
    let reg = reg | mask;
    reg
}

make_reg_get!(
    get_led_error,
    "led",
    LedIndex,
    GetLedErrorResponse,
    REG_EFLAG0,
    get_led_error_reg_from_reg,
    get_led_error_val_from_reg,
    bounds_check_led_index
);
fn get_led_error_reg_from_reg(base_reg: &u8, val: &LedIndex) -> u8 {
    base_reg + (i32::from(val.clone()) / 4) as u8
}
fn get_led_error_val_from_reg(reg: &u8, index: &LedIndex) -> LedError {
    let rshift = (i32::from(index.clone()) % 4) * 2;
    let reg = reg >> rshift;
    let error: u8 = reg & 0b11;
    match error {
        0b00 => LedError::NONE,
        0b01 => LedError::SHORT,
        0b10 => LedError::OPEN,
        0b11 => LedError::DNE,
        _ => {
            assert!(false, "Somehow reg & 0b11 is greater than 3!");
            LedError::NONE
        },
    }
}

macro_rules! handle_pca_error {
    ($val:tt, $rsp_ty:tt) => {
        ok(match $val {
            Ok(x) => x,
            Err(e) => $rsp_ty::OperationFailed(OpError {
                error: Some(match e {
                    I2cHandlingError::PcaError(e) => e,
                    _ => format!("Internal error: {:?}", e).to_string(),
                }),
            }),
        })
    };
}

macro_rules! handle_api_error {
    ($val:tt, $info:tt, $rsp_ty:tt) => {
        match $val {
            ApiError(e) => I2cHandlingError::PcaError(
                format!("Hit ApiError geting {} {:?}", $info, e).to_string(),
            ),
        }
    };
}

pub(crate) fn get_addr_info(
    bus_id: BusId,
    addr: Addr,
    num: AddrIndex,
) -> Box<Future<Item = GetAddrInfoResponse, Error = ApiError>> {
    info!("API get_addr_info : {:?} {:?} {:?}", bus_id, addr, num);
    Box::new(bounds_check!(
        num,
        "num",
        bounds_check_addr_index,
        GetAddrInfoResponse,
        {
            // Compose from other calls
            // We have to pass "info" around from one successful call to the next
            let mut info = AddrInfo::new();
            info.index = Some(i32::from(num.clone()) as u32);
            get_addr_enabled(bus_id.clone(), addr.clone(), num.clone())
                .map_err(|err| handle_api_error!(err, "addr enabled", GetAddrInfoResponse))
                .and_then(move |rsp| match rsp {
                    GetAddrEnabledResponse::OK(x) => {
                        info.enabled = Some(x.clone());
                        ok(GetAddrInfoResponse::OK(info))
                    }
                    _ => failed(I2cHandlingError::PcaError(
                        "Failed to get addr enabled information".to_string(),
                    )),
                })
                .and_then(move |rsp| {
                    let mut info = match rsp {
                        GetAddrInfoResponse::OK(info) => info,
                        _ => {
                            assert!(true, "Invalid arm - only provided OK response above");
                            AddrInfo::new()
                        }
                    };
                    get_addr_value(bus_id, addr, num)
                        .map_err(|err| handle_api_error!(err, "addr value", GetAddrInfoResponse))
                        .and_then(move |x| match x {
                            GetAddrValueResponse::OK(x) => {
                                info.addr = Some(x.clone() as u32);
                                ok(GetAddrInfoResponse::OK(info))
                            }
                            _ => failed(I2cHandlingError::PcaError(
                                "Failed to get addr value information".to_string(),
                            )),
                        })
                })
                .then(|r| handle_pca_error!(r, GetAddrInfoResponse))
                .then(|x| { info!("API get_addr_info -> {:?}", x); x } )
        }
    ))
}

pub(crate) fn get_led_info(
    bus_id: BusId,
    addr: Addr,
    led: LedIndex,
) -> Box<Future<Item = GetLedInfoResponse, Error = ApiError>> {
    info!("API get_led_info : {:?} {:?} {:?}", bus_id, addr, led);
    Box::new(bounds_check!(
        led,
        "led",
        bounds_check_led_index,
        GetLedInfoResponse,
        {
            // Compose from other calls
            // We have to pass "info" around from one successful call to the next
            let mut info = LedInfo::new();
            info.index = Some(i32::from(led.clone()) as u32);
            let bus_id1 = bus_id.clone();
            let bus_id2 = bus_id.clone();
            let bus_id3 = bus_id.clone();
            let addr1 = addr.clone();
            let addr2 = addr.clone();
            let addr3 = addr.clone();
            let led1 = led.clone();
            let led2 = led.clone();
            let led3 = led.clone();
            get_led_state(bus_id.clone(), addr.clone(), led.clone())
                .map_err(|err| handle_api_error!(err, "led state", GetLedInfoResponse))
                .and_then(move |rsp| match rsp {
                    GetLedStateResponse::OK(x) => {
                        info.state = Some(x.clone());
                        ok(GetLedInfoResponse::OK(info))
                    }
                    _ => failed(I2cHandlingError::PcaError(
                        "Failed to get led state information".to_string(),
                    )),
                })
                .and_then(move |rsp| {
                    let mut info = match rsp {
                        GetLedInfoResponse::OK(info) => info,
                        _ => {
                            assert!(true, "Invalid arm - only provided OK response above");
                            LedInfo::new()
                        }
                    };
                    get_led_pwm(bus_id1, addr1, led1)
                        .map_err(|err| handle_api_error!(err, "led pwm", GetLedInfoResponse))
                        .and_then(move |x| match x {
                            GetLedPwmResponse::OK(x) => {
                                info.pwm = Some(x.clone() as u32);
                                ok(GetLedInfoResponse::OK(info))
                            }
                            _ => failed(I2cHandlingError::PcaError(
                                "Failed to get addr pwm information".to_string(),
                            )),
                        })
                })
                .and_then(move |rsp| {
                    let mut info = match rsp {
                        GetLedInfoResponse::OK(info) => info,
                        _ => {
                            assert!(true, "Invalid arm - only provided OK response above");
                            LedInfo::new()
                        }
                    };
                    get_led_current(bus_id2, addr2, led2)
                        .map_err(|err| handle_api_error!(err, "led current", GetLedInfoResponse))
                        .and_then(move |x| match x {
                            GetLedCurrentResponse::OK(x) => {
                                info.current = Some(x.clone() as u32);
                                ok(GetLedInfoResponse::OK(info))
                            }
                            _ => failed(I2cHandlingError::PcaError(
                                "Failed to get addr current information".to_string(),
                            )),
                        })
                })
                .and_then(move |rsp| {
                    let mut info = match rsp {
                        GetLedInfoResponse::OK(info) => info,
                        _ => {
                            assert!(true, "Invalid arm - only provided OK response above");
                            LedInfo::new()
                        }
                    };
                    get_led_error(bus_id3, addr3, led3)
                        .map_err(|err| handle_api_error!(err, "led error", GetLedInfoResponse))
                        .and_then(move |x| match x {
                            GetLedErrorResponse::OK(x) => {
                                info.error = Some(x.clone());
                                ok(GetLedInfoResponse::OK(info))
                            }
                            _ => failed(I2cHandlingError::PcaError(
                                "Failed to get addr error information".to_string(),
                            )),
                        })
                })
                .then(|r| handle_pca_error!(r, GetLedInfoResponse))
                .then(|x| { info!("API get_led_info -> {:?}", x); x } )
        }
    ))
}

// XXX Should maybe do in another thread as we're going to block
// XXX Not handling errors properly - e.g. bad address, maybe handle busId and Addr checking manually?
// XXX If we get an error we're still querying the rest rather than stopping
pub(crate) fn get_led_info_all(
    bus_id: BusId,
    addr: Addr,
) -> Box<Future<Item = GetLedInfoAllResponse, Error = ApiError>> {
    info!("API get_led_info : {:?} {:?}", bus_id, addr);
    let range = 0..24;
    let s = stream::unfold(range.into_iter(), move |mut vals| 
        {
            match vals.next() {
                Some(x) => {
                    Some(get_led_info(bus_id.clone(), addr.clone(), x.into())
                        .map(|rsp| {
                            match rsp {
                                GetLedInfoResponse::OK(x) => (x, vals),
                                GetLedInfoResponse::BadRequest(_) => (LedInfo::new(), vals),
                                GetLedInfoResponse::OperationFailed(_) => (LedInfo::new(), vals),
                            }  
                        })
                    )
                },
                None => None,
            }
        }
    ).collect();
    let rsp = s.wait();
    let rsp = match rsp {
        Ok(rsp) => GetLedInfoAllResponse::OK(rsp),
        Err(e) => GetLedInfoAllResponse::OperationFailed(OpError { error: Some(format!("Failed to collect LED info: {:?}", e))}),
    };
    info!("API get_led_info -> {:?}", rsp);
    Box::new(ok(rsp))
}

/*
GET:
DONE sleep -> MODE1 bit 4, bool
DONE group -> MODE2 bit 5, models::Group::DIM/BLINK
DONE ouput_change -> MODE2 bit 3, models::OutputChange::STOP/ACK
DONE pwm -> GRPPWM -> u32
DONE freq -> GRPFREQ -> u32
DONE offset -> OFFSET -> u32
DONE current -> IREFALL ->u32
DONE addr -> SUBADR1/2/3 -> Vec<models::AdrInfo{ index: Option<u32>, enabled: Option<bool>, addr: Option<u32>}
*/

/*
pub(crate) fn get_config(
    bus_id: BusId,
    addr: Addr,
) -> Box<Future<Item = GetConfigResponse, Error = ApiError>> {
    let bus_id: i32 = bus_id.into();
    let addr: i32 = addr.into();
    let config = Config::new();

    Box::new(



        I2CBUS_HANDLE
            .i2c_bus_read_reg(
                // Read the register containing the current sleep value
                bus_id.into(),
                addr.into(),
                REG_MODE1.into(),
                1.into(), // Read 1 byte
            )
            .then(|x| i2cbus_response_to_ok_err(x)) // Convert read_reg response into a Result<OkData, I2cHandlingError>
            .then(
                // Convert to final response types - ths is where the read register is processed
                |x| handle_i2cbus_ok_err!(x, I2cBusRead, GetConfigResponse, get_config_i2cbus_ok),
            ),
    )
}



fn get_config_i2cbus_ok(rsp: BusRead) -> GetconfigResponse {

XXX

    let old_reg_val = rsp.values[0];
    info!("Read MODE1 register value 0x{:x?}", old_reg_val);
    GetSleepResponse::OK(i32::from(old_reg_val) & 0b1_0000 != 0) // SLEEP is bit 4
}

*/
