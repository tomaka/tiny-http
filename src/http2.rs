use http2_frame::Frame;

struct Connection {
    streams: Vec<Stream>,
}

impl Connection {
    fn inject(&mut self, frame: Frame) {
        match frame {
            Frame::Data { stream_id, end_stream, data } => {
            },
            Frame::Priority { stream_id, exclusive, dependency, weight } => {

            },
            Frame::RstStream { stream_id, error } => {

            },
            Frame::SettingsAck => {

            },
        }
    }
}

struct Stream {
    id: u32,
}

enum StreamState {
    Idle,
    ReservedLocal,
    ReservedRemote,
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
}
