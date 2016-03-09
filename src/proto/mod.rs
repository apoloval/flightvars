//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod oacsp;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Open
}

pub trait ProtocolRead<I> {
    type IntoIter: IntoIterator<Item=Message>;
    fn iter_from(&self, input: I) -> Self::IntoIter;
}

pub struct IdentityProtocolRead;

pub fn id_proto() -> IdentityProtocolRead { IdentityProtocolRead }

impl<I: IntoIterator<Item=Message>> ProtocolRead<I> for IdentityProtocolRead {
    type IntoIter = I;
    fn iter_from(&self, input: I) -> I { input }
}
