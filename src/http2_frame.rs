use nom::IResult;
use nom::Needed;
use nom::Err;
use nom::ErrorKind;
use nom;

pub enum Frame<'a> {
    Data {
        stream_id: u32,
        end_stream: bool,
        data: &'a [u8],
    },
    Priority {
        stream_id: u32,
        exclusive: bool,
        dependency: u32,
        weight: u8,     // between 0 and 255
    },
    RstStream {
        stream_id: u32,
        error: u32,
    },
    SettingsAck,
}

#[repr(u32)]
pub enum Error {
    ProtocolError = 0x1,
    InternalError = 0x2,
    FlowControlError = 0x3,
    SettingsTimeout = 0x4,
    StreamClosed = 0x5,
    FrameSizeError = 0x6,
    RefusedStream = 0x7,
    Cancel = 0x8,
    CompressionError = 0x9,
    ConnectError = 0xa,
    EnhanceYourCalm = 0xb,
    InadequateSecurity = 0xc,
    Http11Required = 0xd,
}

/// Parses an HTTP 2 frame.
///
/// Returns `None` if the frame is of unknown type (the specs says that frames of unknown
/// type must be ignored and discarded).
fn http2_frame(i: &[u8]) -> IResult<&[u8], Option<Frame>, Error> {
    let (i, payload_size) = try_parse!(i, fix_error!(Error, be_24));
    let payload_size = payload_size as usize;

    const FRAME_HEADER_SIZE: usize = 9;
    if i.len() < payload_size + FRAME_HEADER_SIZE - 3 {
        return IResult::Incomplete(Needed::Size(payload_size + FRAME_HEADER_SIZE));
    }

    let (i, frame_ty) = try_parse!(i, fix_error!(Error, nom::be_u8));
    let (i, flags) = try_parse!(i, fix_error!(Error, nom::be_u8));

    let (i, stream_id) = try_parse!(i, fix_error!(Error, nom::be_u32));
    let stream_id = stream_id & 0x7fffffff;     // zeroing the reserved bit

    let (i, frame) = match frame_ty {
        0x0 => try_parse!(i, apply!(http2_frame_data, payload_size, flags, stream_id)),
        0x1 => try_parse!(i, apply!(http2_frame_header, payload_size, flags, stream_id)),
        0x2 => try_parse!(i, apply!(http2_frame_priority, payload_size, flags, stream_id)),
        0x3 => try_parse!(i, apply!(http2_frame_rst_stream, payload_size, flags, stream_id)),
        _ => return IResult::Done(&i[payload_size..], None)
    };

    IResult::Done(i, Some(frame))
}

/// Parses the payload of a DATA frame.
fn http2_frame_data(i: &[u8], size: usize, flags: u8, stream_id: u32) -> IResult<&[u8], Frame, Error> {
    let end_stream = (flags & 0x1) != 0;
    let padded = (flags & 0x8) != 0;

    if i.len() < size {
        return IResult::Incomplete(Needed::Size(size));
    }

    if padded && i.len() < 1 {
        // FIXME: return error instead
        panic!()
    }

    let (i, padding) = if padded {
        try_parse!(i, fix_error!(Error, nom::be_u8))
    } else {
        (i, 0)
    };

    let data_size = match size.checked_sub(padding as usize) {
        Some(size) => size,
        None => panic!()        // FIXME: return error
    };

    IResult::Done(&i[size..], Frame::Data {
        stream_id: stream_id,
        end_stream: end_stream,
        data: &i[..data_size],
    })
}

/// Parses the payload of a HEADER frame.
fn http2_frame_header(i: &[u8], size: usize, flags: u8, stream_id: u32) -> IResult<&[u8], Frame, Error> {
    let end_stream = (flags & 0x1) != 0;
    let end_header = (flags & 0x4) != 0;
    let padded = (flags & 0x8) != 0;
    let priority = (flags & 0x20) != 0;

    unimplemented!()
}

/// Parses the payload of a PRIORITY frame.
fn http2_frame_priority(i: &[u8], size: usize, flags: u8, stream_id: u32) -> IResult<&[u8], Frame, Error> {
    if size != 5 {
        panic!()        // FIXME: return error
    }

    if stream_id == 0 {
        panic!()        // FIXME: return error
    }

    if i.len() < 5 {
        return IResult::Incomplete(Needed::Size(5));
    }

    let (i, dependency) = try_parse!(i, fix_error!(Error, nom::be_u32));
    let (i, weight) = try_parse!(i, fix_error!(Error, nom::be_u8));

    let exclusive = (dependency & 0x80000000) != 0;
    let dependency = dependency & 0x7fffffff;

    IResult::Done(&i[5..], Frame::Priority {
        stream_id: stream_id,
        exclusive: exclusive,
        dependency: dependency,
        weight: weight,
    })
}

/// Parses the payload of a RST_STREAM frame.
fn http2_frame_rst_stream(i: &[u8], size: usize, flags: u8, stream_id: u32) -> IResult<&[u8], Frame, Error> {
    if size != 4 {
        panic!()        // FIXME: return error
    }

    if stream_id == 0 {
        panic!()        // FIXME: return error
    }

    let (i, error) = try_parse!(i, fix_error!(Error, nom::be_u32));

    IResult::Done(&i[4..], Frame::RstStream {
        stream_id: stream_id,
        error: error,
    })
}

/// Parses the payload of a SETTINGS frame.
fn http2_frame_settings(i: &[u8], size: usize, flags: u8, stream_id: u32) -> IResult<&[u8], Frame, Error> {
    if size % 6 != 0 {
        panic!()        // FIXME: return error
    }

    if stream_id != 0 {
        panic!()        // FIXME: return error
    }

    // ack
    if (flags & 0x1) != 0 {
        if size != 0 { panic!() }       // FIXME: return error
        return IResult::Done(i, Frame::SettingsAck);
    }

    /*try_parse!(
        count!(
            chain!()
            ,
        size / 6)
    )

    let (i, error) = try_parse!(i, fix_error!(Error, nom::be_u32));

    IResult::Done(&i[4..], Frame::RstStream {
        error: error,
    })*/

    unimplemented!()
}

/// Parses a big endian 24bits number.
fn be_24(i: &[u8]) -> IResult<&[u8], u32> {
    if i.len() < 3 {
        return IResult::Incomplete(Needed::Size(3));
    }

    let length = ((i[0] as u32) << 16) | ((i[1] as u32) << 8) | (i[2] as u32);
    IResult::Done(&i[3..], length)
}
