// SPDX-License-Identifier: CC0-1.0

//! Bitcoin network-related network messages.
//!
//! This module defines network messages which describe peers and their
//! capabilities.
use std::borrow::Cow;

use bitcoin::consensus::{encode, Decodable, Encodable, ReadExt, WriteExt};
use hashes::sha256d;
use io::{BufRead, Write};

use crate::address::Address;
use crate::consensus::{impl_consensus_encoding, impl_vec_wrapper};
use crate::ServiceFlags;

// Some simple messages

/// The `version` message
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct VersionMessage {
    /// The P2P network protocol version
    pub version: u32,
    /// A bitmask describing the services supported by this node
    pub services: ServiceFlags,
    /// The time at which the `version` message was sent
    pub timestamp: i64,
    /// The network address of the peer receiving the message
    pub receiver: Address,
    /// The network address of the peer sending the message
    pub sender: Address,
    /// A random nonce used to detect loops in the network
    ///
    /// The nonce can be used to detect situations when a node accidentally
    /// connects to itself. Set it to a random value and, in case of incoming
    /// connections, compare the value - same values mean self-connection.
    ///
    /// If your application uses P2P to only fetch the data and doesn't listen
    /// you may just set it to 0.
    pub nonce: u64,
    /// A string describing the peer's software
    pub user_agent: String,
    /// The height of the maximum-work blockchain that the peer is aware of
    pub start_height: i32,
    /// Whether the receiving peer should relay messages to the sender; used
    /// if the sender is bandwidth-limited and would like to support bloom
    /// filtering. Defaults to false.
    pub relay: bool,
}

impl VersionMessage {
    /// Constructs a new `version` message with `relay` set to false
    pub fn new(
        services: ServiceFlags,
        timestamp: i64,
        receiver: Address,
        sender: Address,
        nonce: u64,
        user_agent: String,
        start_height: i32,
    ) -> VersionMessage {
        VersionMessage {
            version: crate::PROTOCOL_VERSION,
            services,
            timestamp,
            receiver,
            sender,
            nonce,
            user_agent,
            start_height,
            relay: false,
        }
    }
}

impl_consensus_encoding!(
    VersionMessage,
    version,
    services,
    timestamp,
    receiver,
    sender,
    nonce,
    user_agent,
    start_height,
    relay
);

/// message rejection reason as a code
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RejectReason {
    /// malformed message
    Malformed = 0x01,
    /// invalid message
    Invalid = 0x10,
    /// obsolete message
    Obsolete = 0x11,
    /// duplicate message
    Duplicate = 0x12,
    /// nonstandard transaction
    NonStandard = 0x40,
    /// an output is below dust limit
    Dust = 0x41,
    /// insufficient fee
    Fee = 0x42,
    /// checkpoint
    Checkpoint = 0x43,
}

impl Encodable for RejectReason {
    fn consensus_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<usize, io::Error> {
        w.write_all(&[*self as u8])?;
        Ok(1)
    }
}

impl Decodable for RejectReason {
    fn consensus_decode<R: BufRead + ?Sized>(r: &mut R) -> Result<Self, encode::Error> {
        Ok(match r.read_u8()? {
            0x01 => RejectReason::Malformed,
            0x10 => RejectReason::Invalid,
            0x11 => RejectReason::Obsolete,
            0x12 => RejectReason::Duplicate,
            0x40 => RejectReason::NonStandard,
            0x41 => RejectReason::Dust,
            0x42 => RejectReason::Fee,
            0x43 => RejectReason::Checkpoint,
            _ => return Err(crate::consensus::parse_failed_error("unknown reject code")),
        })
    }
}

/// Reject message might be sent by peers rejecting one of our messages
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Reject {
    /// message type rejected
    pub message: Cow<'static, str>,
    /// reason of rejection as code
    pub ccode: RejectReason,
    /// reason of rejection
    pub reason: Cow<'static, str>,
    /// reference to rejected item
    pub hash: sha256d::Hash,
}

impl_consensus_encoding!(Reject, message, ccode, reason, hash);

/// A deprecated message type that was used to notify users of system changes. Due to a number of
/// vulerabilities, alerts are no longer used. A final alert was sent as of Bitcoin Core 0.14.0,
/// and is sent to any node that is advertising a potentially vulnerable protocol version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert(Vec<u8>);

impl Alert {
    const FINAL_ALERT: [u8; 96] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 127, 0, 0, 0, 0, 255, 255, 255, 127, 254, 255, 255, 127, 1, 255, 255, 255, 127, 0, 0, 0, 0, 255, 255, 255, 127, 0, 255, 255, 255, 127, 0, 47, 85, 82, 71, 69, 78, 84, 58, 32, 65, 108, 101, 114, 116, 32, 107, 101, 121, 32, 99, 111, 109, 112, 114, 111, 109, 105, 115, 101, 100, 44, 32, 117, 112, 103, 114, 97, 100, 101, 32, 114, 101, 113, 117, 105, 114, 101, 100, 0];

    /// Build the final alert to send to a potentially vulnerable peer.
    pub fn final_alert() -> Self {
        Self(Self::FINAL_ALERT.into())
    }

    /// The final alert advertised by Bitcoin Core. This alert is sent if the advertised protocol
    /// version is vulnerable to the alert-system vulerablities.
    pub fn is_final_alert(&self) -> bool {
        self.0.eq(&Self::FINAL_ALERT)
    }
}

impl_vec_wrapper!(Alert, Vec<u8>);

#[cfg(test)]
mod tests {
    use bitcoin::consensus::encode::{deserialize, serialize};
    use hex_lit::hex;

    use super::*;

    #[test]
    fn version_message_test() {
        // This message is from my satoshi node, morning of May 27 2014
        let from_sat = hex!("721101000100000000000000e6e0845300000000010000000000000000000000000000000000ffff0000000000000100000000000000fd87d87eeb4364f22cf54dca59412db7208d47d920cffce83ee8102f5361746f7368693a302e392e39392f2c9f040001");

        let decode: Result<VersionMessage, _> = deserialize(&from_sat);
        assert!(decode.is_ok());
        let real_decode = decode.unwrap();
        assert_eq!(real_decode.version, 70002);
        assert_eq!(real_decode.services, ServiceFlags::NETWORK);
        assert_eq!(real_decode.timestamp, 1401217254);
        // address decodes should be covered by Address tests
        assert_eq!(real_decode.nonce, 16735069437859780935);
        assert_eq!(real_decode.user_agent, "/Satoshi:0.9.99/".to_string());
        assert_eq!(real_decode.start_height, 302892);
        assert!(real_decode.relay);

        assert_eq!(serialize(&real_decode), from_sat);
    }

    #[test]
    fn reject_message_test() {
        let reject_tx_conflict = hex!("027478121474786e2d6d656d706f6f6c2d636f6e666c69637405df54d3860b3c41806a3546ab48279300affacf4b88591b229141dcf2f47004");
        let reject_tx_nonfinal = hex!("02747840096e6f6e2d66696e616c259bbe6c83db8bbdfca7ca303b19413dc245d9f2371b344ede5f8b1339a5460b");

        let decode_result_conflict: Result<Reject, _> = deserialize(&reject_tx_conflict);
        let decode_result_nonfinal: Result<Reject, _> = deserialize(&reject_tx_nonfinal);

        assert!(decode_result_conflict.is_ok());
        assert!(decode_result_nonfinal.is_ok());

        let conflict = decode_result_conflict.unwrap();
        assert_eq!("tx", conflict.message);
        assert_eq!(RejectReason::Duplicate, conflict.ccode);
        assert_eq!("txn-mempool-conflict", conflict.reason);
        assert_eq!(
            "0470f4f2dc4191221b59884bcffaaf00932748ab46356a80413c0b86d354df05"
                .parse::<sha256d::Hash>()
                .unwrap(),
            conflict.hash
        );

        let nonfinal = decode_result_nonfinal.unwrap();
        assert_eq!("tx", nonfinal.message);
        assert_eq!(RejectReason::NonStandard, nonfinal.ccode);
        assert_eq!("non-final", nonfinal.reason);
        assert_eq!(
            "0b46a539138b5fde4e341b37f2d945c23d41193b30caa7fcbd8bdb836cbe9b25"
                .parse::<sha256d::Hash>()
                .unwrap(),
            nonfinal.hash
        );

        assert_eq!(serialize(&conflict), reject_tx_conflict);
        assert_eq!(serialize(&nonfinal), reject_tx_nonfinal);
    }

    #[test]
    fn alert_message_test() {
        let alert_hex = hex!("60010000000000000000000000ffffff7f00000000ffffff7ffeffff7f01ffffff7f00000000ffffff7f00ffffff7f002f555247454e543a20416c657274206b657920636f6d70726f6d697365642c207570677261646520726571756972656400");
        let alert: Alert = deserialize(&alert_hex).unwrap();
        assert!(alert.is_final_alert());
    }
}
