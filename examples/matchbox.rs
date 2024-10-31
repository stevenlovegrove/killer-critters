use futures::{select, FutureExt};
use futures_timer::Delay;
use log::info;
use matchbox_socket::{PeerState, WebRtcSocket};
use std::time::Duration;

#[tokio::main]
async fn main() {
    info!("Connecting to matchbox");
    let (mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/");

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);

    loop {
        // Handle any new peers
        for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => {
                    info!("Peer joined: {peer}");
                    let packet = "hello friend!".as_bytes().to_vec().into_boxed_slice();
                    socket.send(packet, peer);
                }
                PeerState::Disconnected => {
                    info!("Peer left: {peer}");
                }
            }
        }

        // Accept any messages incoming
        for (peer, packet) in socket.receive() {
            let message = String::from_utf8_lossy(&packet);
            info!("Message from {peer}: {message:?}");
        }

        select! {
            // Restart this loop every 100ms
            _ = (&mut timeout).fuse() => {
                timeout.reset(Duration::from_millis(100));
            }

            // Or break if the message loop ends (disconnected, closed, etc.)
            _ = &mut loop_fut => {
                break;
            }
        }
    }
}
