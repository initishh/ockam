use ockam_transport_tcp::connection::TcpConnection;
use ockam_transport_tcp::error::TransportError;
use ockam_transport_tcp::listener::TcpListener;
use ockam_transport_tcp::traits::Connection;
use rand::prelude::*;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::runtime::Builder;
use tokio::time::Duration;

pub async fn random_worker(mut c: Box<dyn Connection>, text: &str) {
    let mut total = 0f64;
    loop {
        let mut rng = rand::thread_rng();
        let y: f64 = rng.gen(); // generates a float between 0 and 1
        let mut buff = [0u8; 120];
        let sleep = tokio::time::sleep(Duration::from_millis((y * 100.0) as u64));
        tokio::pin!(sleep);
        tokio::select! {
            _ = &mut sleep => {
                let r = c.send(text.as_bytes()).await;
                if r.is_err() {
                    println!("send returned {:?}", r.unwrap());
                    return;
                }
            }
            r = c.receive(&mut buff) => {
                match r {
                    Ok(n) => {
                        println!("{} {}", String::from_utf8(buff[0..n].to_vec()).unwrap(),y);
                    }
                    Err(e) => {
                        if !matches!(e, TransportError::ConnectionClosed) {
                            panic!(format!("{:?}", e));
                        }
                        return;
                    }
                }
            }
        }
        total += y;
        if total >= 5.00 {
            return;
        }
    }
}

#[test]
pub fn test_message_send() {
    let runtime = Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async {
        let socket_addr = std::net::SocketAddr::from_str("127.0.0.1:4050").unwrap();
        let mut m_listener = TcpListener::create(socket_addr.clone()).await.unwrap();
        let mut m_connection_1 =
            TcpConnection::create(SocketAddr::from_str("127.0.0.1:4050").unwrap());

        let m_connection_2 = {
            let f1 = m_connection_1.connect();

            let f2 = m_listener.accept();
            let (r1, r2) = tokio::join!(f1, f2);
            assert!(r1.is_ok(), r2.is_ok());
            r2.unwrap()
        };

        let f_w1 = random_worker(m_connection_1, "ping");
        let f_w2 = random_worker(m_connection_2, "pong");
        tokio::join!(f_w1, f_w2);
    });
}