use crate::cbor::cbor_map;
use crate::error::{URError, URResult};
use crate::impl_template_struct;
use crate::registry_types::UUID;
use crate::traits::{From as FromCbor, MapSize, To};
use crate::types::Bytes;
use alloc::string::ToString;
use alloc::vec::Vec;
use minicbor::data::{Int, Tag};
use minicbor::encode::{Error, Write};
use minicbor::{Decoder, Encoder};

const REQUEST_ID: u8 = 1;
const WITNESS_SET: u8 = 2;

impl_template_struct!(CardanoSignature {
    request_id: Option<Bytes>,
    witness_set: Bytes
});

impl MapSize for CardanoSignature {
    fn map_size(&self) -> u64 {
        let mut size = 1;
        if self.request_id.is_some() {
            size = size + 1;
        }
        size
    }
}

impl<C> minicbor::Encode<C> for CardanoSignature {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.map(self.map_size())?;

        if let Some(id) = &self.request_id {
            e.int(Int::from(REQUEST_ID))?
                .tag(Tag::Unassigned(UUID.get_tag()))?
                .bytes(id)?;
        }

        e.int(Int::from(WITNESS_SET))?
            .bytes(self.get_witness_set().as_ref())?;

        Ok(())
    }
}

impl<'b, C> minicbor::Decode<'b, C> for CardanoSignature {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let mut cardano_signature = CardanoSignature::default();
        cbor_map(d, &mut cardano_signature, |key, obj, d| {
            let key =
                u8::try_from(key).map_err(|e| minicbor::decode::Error::message(e.to_string()))?;
            match key {
                REQUEST_ID => {
                    d.tag()?;
                    obj.set_request_id(Some(d.bytes()?.to_vec()));
                }
                WITNESS_SET => {
                    obj.set_witness_set(d.bytes()?.to_vec());
                }
                _ => {}
            }
            Ok(())
        })?;
        Ok(cardano_signature)
    }
}

impl To for CardanoSignature {
    fn to_bytes(&self) -> URResult<Vec<u8>> {
        minicbor::to_vec(self.clone()).map_err(|e| URError::CborDecodeError(e.to_string()))
    }
}

impl FromCbor<CardanoSignature> for CardanoSignature {
    fn from_cbor(bytes: Vec<u8>) -> URResult<CardanoSignature> {
        minicbor::decode(&bytes).map_err(|e| URError::CborDecodeError(e.to_string()))
    }
}
