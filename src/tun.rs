use std::{
    io::{self, ErrorKind, Read, Write},
    os::fd::AsFd,
};

use crate::sockios::IFIdent;

#[derive(Debug)]
pub struct TUN {
    pub file: std::fs::File,
    pub ident: crate::sockios::IFIdent<String>,
}

#[repr(u16)]
pub enum IFFType {
    TUN = libc::IFF_TUN as _,
    _TAP = libc::IFF_TAP as _,
}

pub fn strncpy_rs(dst: &mut [i8], src: &[u8]) {
    for i in 0..dst.len().min(src.len()) {
        dst[i] = src[i] as _;
    }
}

pub fn cvt(res: i32) -> io::Result<i32> {
    if res >= 0 {
        return Ok(res);
    }
    Err(std::io::Error::from_raw_os_error(-res))
}

pub fn cvt_r<F: FnMut() -> i32>(mut op: F) -> io::Result<i32> {
    loop {
        let res = cvt(op());
        match res {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

crate::define_ioctl!(ioctl_TUNSETIFF, b'T', 202, u32, &mut libc::ifreq);

impl TUN {
    pub fn new(name_fmt: &str) -> std::io::Result<Self> {
        let file = std::fs::File::options()
            .read(true)
            .write(true)
            .open("/dev/net/tun")?;

        let mut request: libc::ifreq = unsafe {
            let mut req: libc::ifreq = std::mem::MaybeUninit::zeroed().assume_init();
            strncpy_rs(req.ifr_name.as_mut_slice(), name_fmt.as_bytes());
            req
        };
        request.ifr_ifru.ifru_flags = IFFType::TUN as i16 | libc::IFF_NO_PI as i16; // | libc::IFF_MULTI_QUEUE as i16;

        let res = cvt_r(|| unsafe { ioctl_TUNSETIFF(file.as_fd(), &mut request) })?;
        println!("{:?}", res);
        unsafe { println!("{:?}", std::ffi::CStr::from_ptr(request.ifr_name.as_ptr())) };
        let if_name = unsafe { std::ffi::CStr::from_ptr(request.ifr_name.as_ptr()) }.to_bytes();
        let if_name = String::from_utf8(if_name.to_vec()).unwrap();

        Ok(Self {
            file,
            ident: IFIdent::new(if_name),
        })
    }

    pub fn send_pkt(&self, pkt_buf: &[u8]) -> io::Result<usize> {
        (&self.file).write(pkt_buf)
    }
    pub fn recv_pkt(&self, pkt_buf: &mut [u8]) -> io::Result<usize> {
        (&self.file).read(pkt_buf)
    }
}
