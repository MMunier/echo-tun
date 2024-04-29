pub const _IOC_NRBITS: u32 = 8;
pub const _IOC_TYPEBITS: u32 = 8;
pub const _IOC_SIZEBITS: u32 = 12;
pub const _IOC_DIRBITS: u32 = 3;

pub const _IOC_NRMASK: u32 = (1 << _IOC_NRBITS) - 1;
pub const _IOC_TYPEMASK: u32 = (1 << _IOC_TYPEBITS) - 1;
pub const _IOC_SIZEMASK: u32 = (1 << _IOC_SIZEBITS) - 1;
pub const _IOC_DIRMASK: u32 = (1 << _IOC_DIRBITS) - 1;

pub const _IOC_NRSHIFT: u32 = 0;
pub const _IOC_TYPESHIFT: u32 = _IOC_NRSHIFT + _IOC_NRBITS;
pub const _IOC_SIZESHIFT: u32 = _IOC_TYPESHIFT + _IOC_TYPEBITS;
pub const _IOC_DIRSHIFT: u32 = _IOC_SIZESHIFT + _IOC_SIZEBITS;

/*
 * Direction bits _IOC_NONE could be 0, but OSF/1 gives it a bit.
 * And this turns out useful to catch old ioctl numbers in header
 * files for us.
 */
pub const _IOC_NONE: u32 = 1;
pub const _IOC_READ: u32 = 2;
pub const _IOC_WRITE: u32 = 4;

#[macro_export]
macro_rules! _IOC {
    ($dir:expr,$typ:literal,$nr:literal,$size:expr) => {
        ((($dir as u32) << $crate::ioctl::_IOC_DIRSHIFT)
            | (($typ as u32) << $crate::ioctl::_IOC_TYPESHIFT)
            | (($nr as u32) << $crate::ioctl::_IOC_NRSHIFT)
            | (($size as u32) << $crate::ioctl::_IOC_SIZESHIFT))
    };
}

#[macro_export]
macro_rules! _IO {
    (typ,nr) => {
        $crate::ioctl::_IOC(_IOC_NONE, (typ), (nr), 0)
    };
}

#[macro_export]
macro_rules! _IOR {
    (typ,nr,size) => {
        crate::ioctl::_IOC(_IOC_READ, (typ), (nr), sizeof(size))
    };
}

#[macro_export]
macro_rules! _IOW {
    ($typ:literal, $nr:literal, $request_args:ty) => {
        $crate::_IOC!(
            $crate::ioctl::_IOC_WRITE,
            $typ,
            $nr,
            core::mem::size_of::<$request_args>()
        )
    };
}

#[macro_export]
macro_rules! _IOWR {
    ($typ:literal, $nr:literal, $request_args:ty) => {
        $crate::ioctl::_IOC(
            _IOC_READ | _IOC_WRITE,
            (typ),
            (nr),
            core::mem::sizeof::<$request_args>(),
        )
    };
}

#[macro_export(local_inner_macros)]
macro_rules! define_ioctl {
    ($name:ident, $ioctl_num:literal, &mut $request_type:ty ) => {
        #[allow(non_snake_case)]
        unsafe fn $name(fd: ::std::os::fd::BorrowedFd, request_args: &mut $request_type) -> i32 {
            unsafe {
                libc::ioctl(
                    ::std::os::fd::AsRawFd::as_raw_fd(&fd),
                    $ioctl_num as _,
                    request_args as *mut _,
                )
            }
        }
    };

    ($name:ident, $ioctl_type:literal, $ioctl_num: literal, $pseudo_type:ty, &mut $request_type:ty ) => {
        #[allow(non_snake_case)]
        unsafe fn $name(fd: ::std::os::fd::BorrowedFd, request_args: &mut $request_type) -> i32 {
            const IOCTL_CONST: u32 = _IOW!($ioctl_type, $ioctl_num, $pseudo_type);
            unsafe {
                libc::ioctl(
                    ::std::os::fd::AsRawFd::as_raw_fd(&fd),
                    IOCTL_CONST as _,
                    request_args as *mut _,
                )
            }
        }
    };
}
