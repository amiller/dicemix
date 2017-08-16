use std::cmp::Ordering;
use std::collections::VecDeque;
use secp256k1::key::PublicKey;
use vec_map::VecMap;
use bit_set::BitSet;

use messages::*;
use super::*;
use self::history::*;
use self::peer::Peer;

mod history;
pub mod peer;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DcPhase {
    Exponential,
    Main,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RunState {
    DcProcess(DcPhase),
    DcReveal(DcPhase),
    Blame,
    Confirm,
}

impl PartialOrd for RunState {
    fn partial_cmp(&self, other: &RunState) -> Option<Ordering> {
        // This is ugly but not uglier than using std::intrinsics::discriminant_value,
        // which does guarantee a proper ordering and would force us to use debug assertions
        // anyway. Note that std::mem::Discriminant<T> does not implement PartialOrd either,
        // because it relies on std::intrinsics::discriminant_value.
        // If this changes in the future, we can replace this function.
        #[inline]
        fn discriminant(x: &RunState) -> u32 {
            match *x {
                RunState::DcProcess(DcPhase::Exponential) => 0,
                RunState::DcReveal(DcPhase::Exponential) => 1,
                RunState::DcProcess(DcPhase::Main) => 2,
                RunState::DcReveal(DcPhase::Main) => 3,
                RunState::Blame => 4,
                RunState::Confirm => 5,
            }
        }

        match (*self, *other) {
            (RunState::Blame, RunState::Confirm) => None,
            (RunState::Confirm, RunState::Blame) => None,
            _ => discriminant(self).partial_cmp(&discriminant(other)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunStateMachine {
    run: RunCounter,
    state: RunState,
    kepks: VecMap<PublicKey>,
    received: BitSet,
    otvk_hashes: Option<Vec<[u8; 32]>>,
    peers_before_dc_exponential: Option<BitSet>,
    peers_before_dc_main: Option<BitSet>,
}

impl RunStateMachine {
    fn new(&mut self, run: RunCounter, kepks: VecMap<PublicKey>) -> Self {
        let num_peers = kepks.len();
        let new = Self {
            run: 0,
            state: RunState::DcProcess(DcPhase::Exponential),
            kepks: kepks,
            received: BitSet::with_capacity(num_peers),
            otvk_hashes: None,
            peers_before_dc_exponential: None,
            peers_before_dc_main: None,
        };
        debug_assert!(new.consistent());
        new
    }

    #[inline]
    fn num_peers(&self) -> usize {
        self.kepks.len()
    }

    #[inline]
    fn set_state(&mut self, state: RunState) {
        assert!(self.state < state);
        self.state = state;
    }

    #[inline]
    fn consistent(&self) -> bool {
        // TODO check also the state ...
        match (self.peers_before_dc_exponential.is_some(),
               self.otvk_hashes.is_some(),
               self.peers_before_dc_main.is_some()) {
            (false, false, false) => true,
            (false, _, _) => false,
            (true, x, y) => x == y,
        }
    }

    fn process_message(&mut self, msg: Message) -> Option<Payload> {
        // The message has a correct signature and is intended for this state of this session.
        // So we can record it.
        let first_from_peer = self.received.insert(msg.header.peer_index as usize);

        // Reject the message if we have recorded a message from this peer already.
        if !first_from_peer {
            return None;
        }
        self.process_payload(msg.payload)
    }

    fn process_payload(&self, payload: Payload) -> Option<Payload> {
        match(self.state, payload) {
            (RunState::DcProcess(DcPhase::Exponential), Payload::DcExponential(msg)) => {
                unimplemented!()
            },
            (RunState::DcProcess(DcPhase::Main), Payload::DcMain(msg)) => {
                unimplemented!()
            },
            (RunState::DcReveal(phase), Payload::Reveal(msg)) => {
                unimplemented!()
            },
            (RunState::Blame, Payload::Blame(msg)) => {
                unimplemented!()
            },
            (RunState::Confirm, Payload::Confirm(msg)) => {
                unimplemented!()
            },
            _ => {
                // TODO Kick the peer out
                unimplemented!()
            }
        }
        assert!(self.consistent());
    }
}

