//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use proto;

pub trait RequestDelivery {
    fn deliver(&mut self, req: proto::DomainRequest);
}

impl RequestDelivery for proto::DomainRequestSender {
    fn deliver(&mut self, req: proto::DomainRequest) {
        self.send(req).unwrap()
    }
}

pub struct DomainRouter;

impl DomainRouter {
}
