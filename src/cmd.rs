use crate::rtt_print;
use ellocopo::Error;
use crate::pld::{PldPWMComp, PldPWMValve, PldAfeCmd_low, PldAfeCmd_high};

pub mod built_info {
   include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

static mut INF_CUR : u8 = 0x20;
static mut RED_CUR : u8 = 0x20;

#[no_mangle]
pub fn cb_erase_hard() -> Result<(), Error> {
    unimplemented!()
}

#[no_mangle]
pub fn cb_write_hard_comp(v : u16) -> Result<(), Error> {
    PldPWMComp::get().write(v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_hard_comp() -> Result<u16, Error> {
    Ok(PldPWMComp::get().read())
}

#[no_mangle]
pub fn cb_write_hard_valv(v : u16) -> Result<(), Error> {
    PldPWMValve::get().write(v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_hard_valv() -> Result<u16, Error> {
    Ok(PldPWMValve::get().read())
}

#[no_mangle]
pub fn cb_write_hard_red(v : u8) -> Result<(), Error> {
    unsafe { RED_CUR = v; }
    let tmp : u16 = unsafe { ((RED_CUR as u16) << 8) | INF_CUR as u16 };
    PldAfeCmd_low::get().write(tmp);
    PldAfeCmd_high::get().write(0x2202);
    Ok(())
}

#[no_mangle]
pub fn cb_read_hard_red() -> Result<u8, Error> {
    Ok(unsafe { RED_CUR })
}

#[no_mangle]
pub fn cb_write_hard_inf(v : u8) -> Result<(), Error> {
    unsafe { INF_CUR = v; }
    let tmp : u16 = unsafe { ((RED_CUR as u16) << 8) | INF_CUR as u16 };
    PldAfeCmd_low::get().write(tmp);
    PldAfeCmd_high::get().write(0x2202);
    Ok(())
}

#[no_mangle]
pub fn cb_read_hard_inf() -> Result<u8, Error> {
    Ok(unsafe { INF_CUR })
}

#[no_mangle]
pub fn cb_erase_info() -> Result<(), Error> {
    unimplemented!()
}

#[no_mangle]
pub fn cb_write_info_version(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_info_version() -> Result<&'static str, Error> {
    Ok(built_info::PKG_VERSION)
}

#[no_mangle]
pub fn cb_erase_build() -> Result<(), Error> {
    unimplemented!()
}

#[no_mangle]
pub fn cb_write_build_version(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_version() -> Result<&'static str, Error> {
     Ok(built_info::PKG_VERSION)
}

#[no_mangle]
pub fn cb_write_build_compiler(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_compiler() -> Result<&'static str, Error> {
     Ok(built_info::RUSTC_VERSION)
}

#[no_mangle]
pub fn cb_write_build_git(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_git() -> Result<&'static str, Error> {
    Ok(built_info::GIT_VERSION.unwrap_or("None"))
}

#[no_mangle]
pub fn cb_write_build_time(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_time() -> Result<&'static str, Error> {
    Ok(built_info::BUILT_TIME_UTC)
}

#[no_mangle]
pub fn cb_write_build_target(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_target() -> Result<&'static str, Error> {
    Ok(built_info::TARGET)
}

#[no_mangle]
pub fn cb_write_build_host(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_host() -> Result<&'static str, Error> {
    Ok(built_info::HOST)
}

#[no_mangle]
pub fn cb_write_build_profile(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_profile() -> Result<&'static str, Error> {
    Ok(built_info::PROFILE)
}

#[no_mangle]
pub fn cb_write_build_authors(_v : &'static str) -> Result<(), Error> {
    Err(Error::NonWriteable)
}

#[no_mangle]
pub fn cb_read_build_authors() -> Result<&'static str, Error> {
    Ok(built_info::PKG_AUTHORS)
}



