use std::{
    io,
    net::{IpAddr, Ipv4Addr},
    ops::{Deref, DerefMut},
    os::fd::{AsFd, FromRawFd, OwnedFd},
};

use crate::tun::{cvt_r, strncpy_rs};

// enum IFRequestType {
//     SIOCSIFFLAGS,
//     SIOCGIFFLAGS,

//     SIOCGIFADDR,
//     SIOCSIFADDR,
//     SIOCDIFADDR,

//     SIOCGIFNETMASK,
//     SIOCSIFNETMASK,
// }

crate::define_ioctl!(ioctl_SIOCSIFFLAGS, 0x8914, &mut libc::ifreq);
crate::define_ioctl!(ioctl_SIOCGIFFLAGS, 0x8913, &mut libc::ifreq);

// crate::define_ioctl!(ioctl_SIOCGIFADDR, 0x8915, &mut libc::ifreq); /* get PA address		*/
crate::define_ioctl!(ioctl_SIOCSIFADDR, 0x8916, &mut libc::ifreq); /* set PA address		*/

// crate::define_ioctl!(ioctl_SIOCGIFNETMASK, 0x891b, &mut libc::ifreq); /* get network PA mask		*/
crate::define_ioctl!(ioctl_SIOCSIFNETMASK, 0x891c, &mut libc::ifreq); /* set network PA mask		*/

fn to_sockaddr_in(ipv4_addr: Ipv4Addr) -> libc::sockaddr_in {
    let mut sockaddr: libc::sockaddr_in = unsafe { core::mem::zeroed() };
    sockaddr.sin_family = libc::AF_INET as _;
    sockaddr.sin_addr = unsafe { core::mem::transmute(ipv4_addr) };
    sockaddr
}

pub struct IFConfigHandle<T: AsRef<str>> {
    socket: SockioHandle,
    ident: IFIdent<T>,
}

impl<T> IFConfigHandle<T>
where
    T: AsRef<str>,
{
    pub fn new(if_ident: T) -> Self {
        Self {
            socket: SockioHandle::new(),
            ident: IFIdent::new(if_ident),
        }
    }

    pub fn set_if_flags(&self, flags: u16) -> io::Result<()> {
        let mut req = IFReq::with_if_name(&self.ident);
        req.0.ifr_ifru.ifru_flags = flags as _;
        cvt_r(|| unsafe { ioctl_SIOCSIFFLAGS(self.socket.as_fd(), &mut req.0) })?;
        Ok(())
    }

    pub fn get_if_flags(&self) -> io::Result<u16> {
        let mut req = IFReq::with_if_name(&self.ident);
        cvt_r(|| unsafe { ioctl_SIOCGIFFLAGS(self.socket.as_fd(), &mut req.0) })?;
        Ok(unsafe { req.0.ifr_ifru.ifru_flags } as _)
    }

    pub fn set_if_addr(&self, addr: IpAddr) -> io::Result<()> {
        match addr {
            IpAddr::V6(_addrv6) => unimplemented!(),
            IpAddr::V4(addrv4) => {
                let mut req = IFReq::with_if_name(&self.ident);
                req.0.ifr_ifru.ifru_addr = unsafe { core::mem::transmute(to_sockaddr_in(addrv4)) };
                cvt_r(|| unsafe { ioctl_SIOCSIFADDR(self.socket.as_fd(), &mut req.0) })?;
            }
        }

        Ok(())
    }

    pub fn set_if_netmask(&self, netmask: IpAddr) -> io::Result<()> {
        match netmask {
            IpAddr::V6(_addrv6) => unimplemented!(),
            IpAddr::V4(addrv4) => {
                let mut req = IFReq::with_if_name(&self.ident);
                req.0.ifr_ifru.ifru_addr = unsafe { core::mem::transmute(to_sockaddr_in(addrv4)) };
                cvt_r(|| unsafe { ioctl_SIOCSIFNETMASK(self.socket.as_fd(), &mut req.0) })?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct SockioHandle(OwnedFd);
impl SockioHandle {
    fn new() -> Self {
        let res =
            cvt_r(|| unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_UDP) })
                .expect("Failed to open socket");
        Self(unsafe { OwnedFd::from_raw_fd(res) })
    }
}

impl Deref for SockioHandle {
    type Target = OwnedFd;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SockioHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct IFIdent<T: AsRef<str>>(T);
impl<T> IFIdent<T>
where
    T: AsRef<str>,
{
    pub fn new(name: T) -> Self {
        assert!(name.as_ref().as_bytes().len() < libc::IFNAMSIZ);
        Self(name)
    }
}

impl<T> Deref for IFIdent<T>
where
    T: AsRef<str>,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct IFReq(libc::ifreq);
impl Default for IFReq {
    fn default() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }
}

impl IFReq {
    fn with_if_name<T: AsRef<str>>(ident: &IFIdent<T>) -> Self {
        let mut req = Self::default();
        strncpy_rs(&mut req.0.ifr_name, ident.0.as_ref().as_bytes());
        req
    }
}
