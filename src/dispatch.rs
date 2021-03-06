// Copyright 2015-2020 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::bus::MessageBus;
use crate::inthost::{Invocation, WasccEntity};
use std::{error::Error, sync::Arc};

use wascap::prelude::KeyPair;
use wascc_codec::capabilities::Dispatcher;

/// A dispatcher is given to each capability provider, allowing it to send
/// commands in to the guest module and await replies. This dispatch
/// is one way, and is _not_ used for the guest module to send commands to capabilities
#[derive(Clone)]
pub(crate) struct WasccNativeDispatcher {
    bus: Arc<MessageBus>,
    capid: String,
    binding: String,
    hk: Arc<KeyPair>,
}

impl WasccNativeDispatcher {
    pub fn new(hk: Arc<KeyPair>, bus: Arc<MessageBus>, capid: &str, binding: &str) -> Self {
        WasccNativeDispatcher {
            bus,
            capid: capid.to_string(),
            binding: binding.to_string(),
            hk,
        }
    }
}

impl Dispatcher for WasccNativeDispatcher {
    /// Called by a capability provider to invoke a function on an actor
    fn dispatch(&self, actor: &str, op: &str, msg: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        trace!(
            "Dispatching operation '{}' ({} bytes) to actor",
            op,
            msg.len()
        );
        let inv = Invocation::new(
            &self.hk,
            WasccEntity::Capability {
                capid: self.capid.to_string(),
                binding: self.binding.to_string(),
            },
            WasccEntity::Actor(actor.to_string()),
            op,
            msg.to_vec(),
        );
        let tgt_sub = crate::bus::actor_subject(actor);
        let resp = self.bus.invoke(&tgt_sub, inv);

        match resp {
            Ok(r) => match r.error {
                Some(e) => Err(format!("Invocation failure: {}", e).into()),
                None => Ok(r.msg),
            },
            Err(e) => Err(Box::new(e)),
        }
    }
}
