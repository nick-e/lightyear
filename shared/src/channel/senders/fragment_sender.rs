use crate::packet::message::{FragmentData, SingleData};
use crate::packet::packet::FRAGMENT_SIZE;
use crate::packet::wrapping_id::MessageId;
use crate::{BitSerializable, MessageContainer, ReadBuffer, ReadWordBuffer};
use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use tracing::trace;

/// `FragmentReceiver` is used to reconstruct fragmented messages
pub(crate) struct FragmentSender {
    pub(crate) fragment_size: usize,
}

impl FragmentSender {
    pub fn new() -> Self {
        Self {
            // TODO: make this overridable?
            fragment_size: FRAGMENT_SIZE,
        }
    }
    pub fn build_fragments(
        &self,
        fragment_message_id: MessageId,
        fragment_bytes: Bytes,
    ) -> Vec<FragmentData> {
        if fragment_bytes.len() < FRAGMENT_SIZE {
            panic!("Fragment size must be at least {}", FRAGMENT_SIZE);
        }
        let chunks = fragment_bytes.chunks(self.fragment_size);
        let num_fragments = chunks.len();
        chunks
            .enumerate()
            // TODO: ideally we don't clone here but we take ownership of the output of writer
            .map(|(fragment_index, chunk)| FragmentData {
                message_id: fragment_message_id,
                fragment_id: fragment_index as u8,
                num_fragments: num_fragments as u8,
                bytes: fragment_bytes.slice_ref(chunk),
            })
            .collect::<_>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::packet::FRAGMENT_SIZE;
    use crate::packet::wrapping_id::MessageId;
    use bytes::Bytes;

    #[test]
    fn test_build_fragments() {
        let message_id = MessageId(0);
        const NUM_BYTES: usize = (FRAGMENT_SIZE as f32 * 2.5) as usize;
        let bytes = Bytes::from(vec![0; NUM_BYTES]);

        let mut sender = FragmentSender::new();

        let fragments = sender.build_fragments(message_id, bytes.clone());
        let expected_num_fragments = 3;
        assert_eq!(fragments.len(), expected_num_fragments);
        assert_eq!(
            fragments.get(0).unwrap(),
            &FragmentData {
                message_id,
                fragment_id: 0,
                num_fragments: expected_num_fragments as u8,
                bytes: bytes.slice(0..FRAGMENT_SIZE),
            }
        );
        assert_eq!(
            fragments.get(1).unwrap(),
            &FragmentData {
                message_id,
                fragment_id: 1,
                num_fragments: expected_num_fragments as u8,
                bytes: bytes.slice(FRAGMENT_SIZE..2 * FRAGMENT_SIZE),
            }
        );
        assert_eq!(
            fragments.get(2).unwrap(),
            &FragmentData {
                message_id,
                fragment_id: 2,
                num_fragments: expected_num_fragments as u8,
                bytes: bytes.slice(2 * FRAGMENT_SIZE..),
            }
        );
    }
}