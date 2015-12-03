// Copyright 2015 The tiny-http Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use util::RefinedTcpStream;
use http1::Http1Client;
use Request;

/// A ClientConnection is an object that will store a socket to a client
/// and return Request objects.
pub struct ClientConnection {
    inner: ClientConnectionInner
}

impl ClientConnection {
    /// Creates a new ClientConnection that takes ownership of the TcpStream.
    pub fn new(write_socket: RefinedTcpStream, mut read_socket: RefinedTcpStream)
               -> ClientConnection
    {
        ClientConnection {
            inner: ClientConnectionInner::Http1(Http1Client::new(write_socket, read_socket))
        }
    }
}

enum ClientConnectionInner {
    Http1(Http1Client),
    Http2,      // TODO: 
}

impl Iterator for ClientConnection {
    type Item = Request;

    /// Blocks until the next request is available.
    ///
    /// Returns None when no new requests will come from the client.
    fn next(&mut self) -> Option<Request> {
        match self.inner {
            ClientConnectionInner::Http1(ref mut client) => client.next(),
            ClientConnectionInner::Http2 => unimplemented!()
        }
    }
}
