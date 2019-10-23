use crate::rtt_print;
use ellocopo::Error;

#[no_mangle]
pub fn cb_erase_name() -> Result<(), Error> {
    rtt_print!("cb_erase_name");
    Ok(())
}

#[no_mangle]
pub fn cb_write_name_test(v: i32) -> Result<(), Error> {
    rtt_print!("cb_write_name_test: {}", v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_name_test() -> Result<i32, Error> {
    rtt_print!("cb_read_name_test");
    Ok(-2777)
}

#[no_mangle]
pub fn cb_write_name_test2(v: u8) -> Result<(), Error> {
    rtt_print!("cb_write_name_test2: {}", v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_name_test2() -> Result<u8, Error> {
    rtt_print!("cb_read_name_test2");
    Ok(42)
}

#[no_mangle]
pub fn cb_write_name_test3(v: &'static str) -> Result<(), Error> {
    rtt_print!("cb_write_name_test3: {}", v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_name_test3() -> Result<&'static str, Error> {
    rtt_print!("cb_read_name_test3");
    Ok("cb_read_name_test3")
}

#[no_mangle]
pub fn cb_erase_survey() -> Result<(), Error> {
    rtt_print!("cb_erase_survey");
    Ok(())
}

#[no_mangle]
pub fn cb_write_survey_name(v: &'static [u8]) -> Result<(), Error> {
    rtt_print!("cb_write_survey_name : {:?}", v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_survey_name() -> Result<&'static [u8], Error> {
    rtt_print!("cb_read_survey_name");
    Ok(&[0xA5, 0xA5])
}

#[no_mangle]
pub fn cb_write_survey_fam(v: i8) -> Result<(), Error> {
    rtt_print!("cb_write_survey_fam: {}", v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_survey_fam() -> Result<i8, Error> {
    rtt_print!("cb_read_survey_fam");
    Ok(-5)
}

#[no_mangle]
pub fn cb_erase_info() -> Result<(), Error> {
    rtt_print!("cb_erase_info");
    Ok(())
}

#[no_mangle]
pub fn cb_write_info_lal(v: &'static str) -> Result<(), Error> {
    rtt_print!("cb_write_info_lal: {}", v);
    Ok(())
}

#[no_mangle]
pub fn cb_read_info_lal() -> Result<&'static str, Error> {
    rtt_print!("cb_read_info_lal");
    Ok("cb_read_info_lal")
}
