
use phf::phf_map;
use super::rtt_print;

static mut TEST_REG : u32 = 0;


#[repr(packed)]
#[repr(C)]
struct CmdFormatIn {
    sign : u8,
    op : Operation,
    name_sz : u8,
    value_sz : u8,
    payload : [u8; 0x40 - 4],
}

#[repr(packed)]
#[repr(C)]
struct CmdFormatOut {
    sign : u8,
    op : Operation,
    name_sz : u8,
    value_sz : u8,
    payload : [u8; 0x40 - 4],
}


pub fn parse_n_answer(buf : &mut[u8; 0x40]) -> usize {

    let cmd: &mut CmdIn = unsafe { core::mem::transmute(buf.as_mut_ptr()) };
    let name: &str = unsafe { core::str::from_utf8_unchecked(&cmd.payload[0 .. cmd.name_sz as usize])};
    let value : &u32 = unsafe { core::mem::transmute(cmd.payload.as_ptr().offset(cmd.name_sz as isize) )};
    match dispatch_blocking(cmd.op, name, value) {
        Ok(RegType::u32V(value)) => {
            cmd.payload[cmd.name_sz as usize + 0] = value as u8;
            cmd.payload[cmd.name_sz as usize + 1] = (value >> 8) as u8;
            cmd.payload[cmd.name_sz as usize + 2] = (value >> 16) as u8;
            cmd.payload[cmd.name_sz as usize + 3] = (value >> 24) as u8;
            cmd.value_sz = 4;
        }
        Err(Error::Ok) => {
            cmd.value_sz = 0;
        }
        _ => unimplemented!(),
    };

    (4 + cmd.name_sz + cmd.value_sz) as usize
}

pub fn dispatch_blocking(op : Operation, name : &str, value : &u32) -> CmdResult {

    let e_num = STR_TO_ENUM.get(name).ok_or(Error::NoSuch)?;
    let e = &REGISTRY[*e_num as usize];

    match (op, e) {
        (Operation::Erase, Entity::Section) => {
            rtt_print!("Erasing section {}", name);
            Err(Error::Ok)
        }
        (Operation::Read, Entity::Register(_r)) => {
            rtt_print!("Reading reg {}", name);
            let r = unsafe { TEST_REG };
            Ok(RegType::u32V(r))
        }
        (Operation::Write, Entity::Register(_r)) => {
            rtt_print!("Writing reg {}, value {}", name, value);
            unsafe { TEST_REG = *value };
            Err(Error::Ok)
        }
        (_, _) => return Err(Error::WrongOperation),
    }
}

/// Types definition

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Read = 0,
    Write,
    Erase,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Answer {
    Read = 0,
    Write,
    Erase,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum RegType {
    u32V(u32),
    u8V(u8),
}

pub enum Error {
    Ok = 0,
    WrongOperation,
    BadFormat,
    NoSuch,
    NonWriteable,
    Locked,
    EraseNeeded,
}

pub type CmdResult = Result<RegType, Error>;

#[derive(Clone, Copy, Debug)]
enum Entity {
    Section,
    Register(Register),
}

#[derive(Clone, Copy, Debug)]
struct Register {
    dummy : u32,
}

pub fn str_to_enum(s : &'static str) {
    let c = STR_TO_ENUM.get(s);
    rtt_print!("{:?}",  &c);
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));