use crate::tun::TUN;
use crate::sockios::IFConfigHandle;

mod ioctl;
mod sockios;
mod tun;

fn outgoing_thread(tun: &TUN, sender: std::sync::mpsc::Sender<Vec<u8>>) {
    loop {
        let mut buf = [0u8; 0x800];
        let bytes_read = tun.recv_pkt(&mut buf).expect("Failed to receive pkt!");        
        sender.send(buf[..bytes_read].to_vec()).unwrap();
        
        println!("read {bytes_read} bytes:");
        println!("    {}", buf[..bytes_read].escape_ascii());
    }
}

fn incoming_thread(tun: &TUN, recv: std::sync::mpsc::Receiver<Vec<u8>>) {
    loop {
        let mut pkt = recv.recv().unwrap();
        if &pkt[20..22] != &[8u8, 0u8] {
            continue;
        }

        let mut dst: [u8; 4] = pkt[16..20].try_into().unwrap();
        pkt[12..16].swap_with_slice(&mut dst);
        pkt[16..20].copy_from_slice(dst.as_slice());
        pkt[20..22].copy_from_slice(&[0u8, 0u8]);
        icmp_chksum(&mut pkt[20..]);
        
        tun.send_pkt(pkt.as_slice()).expect("Failed to send packet");
        println!("got echo pkt!");
    }
}

fn icmp_chksum(pkt: &mut [u8]) {
    pkt[2] = 0;
    pkt[3] = 0;
    let mut chksum = pkt.chunks_exact(2).fold(0u32, |acc, x| {
        acc.wrapping_add(u16::from_le_bytes(x.try_into().unwrap()) as _)
    });

    if pkt.len() & 1 != 0 {
        chksum = chksum.wrapping_add(*pkt.last().unwrap() as _);
    }

    let chksum: u16 = chksum.wrapping_add(chksum >> 16) as u16 ^ 0xFFFFu16;
    let chksum = chksum.to_le_bytes();
    pkt[2] = chksum[0];
    pkt[3] = chksum[1];
}

fn main() {
    let tun = TUN::new("echo%d").unwrap();
    println!("{:?}", tun);

    let if_config = IFConfigHandle::new(tun.ident.as_str());
    if_config
        .set_if_addr("203.0.113.0".parse().unwrap())
        .unwrap();
    if_config
        .set_if_netmask("255.255.255.254".parse().unwrap())
        .unwrap();

    let flags = if_config.get_if_flags().unwrap();
    if_config
        .set_if_flags(flags | (libc::IFF_UP as u16))
        .unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::scope(|_scope| {
        _scope.spawn(|| outgoing_thread(&tun, sender));
        _scope.spawn(|| incoming_thread(&tun, receiver));
    });
}
