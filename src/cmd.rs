
use phf::phf_map;
use super::rtt_print;

const KEYWORDS : phf::Map<&'static str, RegNum> = phf_map! {
    "hard"      => RegNum::hard,
    "hard/name" => RegNum::hard_name,
    "hard/num"  => RegNum::hard_num,
    "super/long/name/tarta/tatat/tatata"  => RegNum::hard_info,
};

#[derive(Clone, Copy, Debug)]
pub struct Test {
    a : u32,
    b : u8,
    c : u16,
}

pub const PATH: phf::Map<&'static str, Test> = phf_map! {
    "hard"      => Test { a : 1, b : 2, c : 3},
    "hard/name" => Test { a : 2, b : 3, c : 4},
    "hard/num"  => Test { a : 5, b : 5, c : 5},
    "super/long/name/tarta/tatat/tatata"  => Test { a : 6, b : 6, c : 6},
};

enum Error {
    Ok = 0,
    BadFormat,
    NoSuch,
    NonWriteable,
    Locked,
    EraseNeeded,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum RegNum {
    hard,
    hard_name,
    hard_num,
    hard_info,
    Max,
}

pub fn str_to_enum(s : &'static str) {
    let c = KEYWORDS.get(s);
    rtt_print!("{:?}",  &c);
}

pub fn str_to_struct(s : &'static str) {
    let c = PATH.get(s);
    rtt_print!("{:?}",  &c);
}