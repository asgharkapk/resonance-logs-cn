//! Lifecycle-managed capture-envelope decoder.

use crate::live::protocol::decoder::ProtocolDecoder;
use crate::live::runtime::events::{CaptureEnvelope, ProtocolBatch};
use log::info;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const CHANNEL_POLL_INTERVAL: Duration = Duration::from_millis(1);

pub struct DecodeWorkerHandle {
    cancellation: CancellationToken,
    join: Option<JoinHandle<()>>,
}

impl Drop for DecodeWorkerHandle {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

impl DecodeWorkerHandle {
    /// Waits for a natural, draining shutdown. This deliberately does not
    /// cancel; the input channel must be closed by stopping capture first.
    pub fn join(mut self) -> std::thread::Result<()> {
        self.join
            .take()
            .expect("decode join handle is present")
            .join()
    }
}

/// Spawns the only protobuf decoder. Every accepted capture envelope produces
/// exactly one protocol batch, including unsupported and malformed packets.
pub fn spawn_decode_worker(
    mut input: mpsc::Receiver<CaptureEnvelope>,
    output: mpsc::Sender<ProtocolBatch>,
) -> DecodeWorkerHandle {
    let cancellation = CancellationToken::new();
    let worker_cancellation = cancellation.clone();
    let join = std::thread::Builder::new()
        .name("protocol-decoder".to_string())
        .spawn(move || {
            let mut decoder = ProtocolDecoder::new();
            loop {
                if worker_cancellation.is_cancelled() {
                    break;
                }
                match input.try_recv() {
                    Ok(envelope) => {
                        let batch = decoder.decode(envelope);
                        if !send_with_backpressure(&output, &worker_cancellation, batch) {
                            break;
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        std::thread::sleep(CHANNEL_POLL_INTERVAL);
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
                }
            }
            info!(target: "app::live", "protocol decoder worker exiting");
        })
        .expect("failed to spawn protocol decoder thread");

    DecodeWorkerHandle {
        cancellation,
        join: Some(join),
    }
}

fn send_with_backpressure(
    output: &mpsc::Sender<ProtocolBatch>,
    cancellation: &CancellationToken,
    mut batch: ProtocolBatch,
) -> bool {
    loop {
        if cancellation.is_cancelled() {
            return false;
        }
        match output.try_send(batch) {
            Ok(()) => {
                return true;
            }
            Err(mpsc::error::TrySendError::Full(returned)) => {
                batch = returned;
                std::thread::sleep(CHANNEL_POLL_INTERVAL);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::runtime::events::{PacketDirection, PacketKey};

    fn envelope(sequence: u64) -> CaptureEnvelope {
        CaptureEnvelope {
            capture_sequence: sequence,
            stream_id: 7,
            stream_epoch: 1,
            captured_wall_ms: 1_000,
            captured_mono_ns: sequence * 1_000_000,
            direction: PacketDirection::ServerToClient,
            key: PacketKey {
                opcode: 123_456,
                service_id: None,
                method_id: None,
            },
            payload: bytes::Bytes::new(),
        }
    }

    #[test]
    fn closed_input_drains_all_batches_before_join() {
        let (input_tx, input_rx) = mpsc::channel(2);
        let (output_tx, mut output_rx) = mpsc::channel(2);
        input_tx.blocking_send(envelope(1)).expect("first input");
        input_tx.blocking_send(envelope(2)).expect("second input");
        drop(input_tx);

        let worker = spawn_decode_worker(input_rx, output_tx);
        worker.join().expect("worker joins");

        assert_eq!(output_rx.blocking_recv().unwrap().meta.capture_sequence, 1);
        assert_eq!(output_rx.blocking_recv().unwrap().meta.capture_sequence, 2);
    }
}
