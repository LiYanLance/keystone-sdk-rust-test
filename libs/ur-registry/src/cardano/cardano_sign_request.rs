use crate::cardano::cardano_cert_key::CardanoCertKey;
use crate::cardano::cardano_utxo::CardanoUTXO;
use crate::cbor::{cbor_array, cbor_map};
use crate::crypto_key_path::CryptoKeyPath;
use crate::error::{URError, URResult};
use crate::registry_types::{RegistryType, CARDANO_SIGN_REQUEST, CRYPTO_KEYPATH, UUID, CARDANO_UTXO, CARDANO_CERT_KEY};
use crate::traits::{From as FromCbor, RegistryItem, To};
use crate::types::Bytes;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use minicbor::data::{Int, Tag};
use minicbor::encode::{Error, Write};
use minicbor::{Decoder, Encoder};

const REQUEST_ID: u8 = 1;
const SIGN_DATA: u8 = 2;
const UTXOS: u8 = 3;
const CERT_KEYS: u8 = 4;
const ORIGIN: u8 = 5;

#[derive(Debug, Clone, Default)]
pub struct CardanoSignRequest {
    request_id: Option<Bytes>,
    sign_data: Bytes,
    utxos: Vec<CardanoUTXO>,
    cert_keys: Vec<CardanoCertKey>,
    origin: Option<String>,
}

impl CardanoSignRequest {
    pub fn default() -> Self {
        Default::default()
    }
    pub fn set_request_id(&mut self, id: Option<Bytes>) {
        self.request_id = id;
    }
    pub fn set_sign_data(&mut self, data: Bytes) {
        self.sign_data = data
    }
    pub fn set_utxos(&mut self, utxos: Vec<CardanoUTXO>) {
        self.utxos = utxos;
    }
    pub fn set_cert_keys(&mut self, cert_keys: Vec<CardanoCertKey>) {
        self.cert_keys = cert_keys;
    }
    pub fn set_origin(&mut self, origin: Option<String>) {
        self.origin = origin;
    }
    pub fn get_request_id(&self) -> Option<Bytes> {
        self.request_id.clone()
    }
    pub fn get_sign_data(&self) -> Bytes {
        self.sign_data.clone()
    }
    pub fn get_utxos(&self) -> Vec<CardanoUTXO> {
        self.utxos.clone()
    }
    pub fn get_cert_keys(&self) -> Vec<CardanoCertKey> {
        self.cert_keys.clone()
    }
    pub fn get_origin(&self) -> Option<String> {
        self.origin.clone()
    }

    pub fn new(
        request_id: Option<Bytes>,
        sign_data: Bytes,
        utxos: Vec<CardanoUTXO>,
        cert_keys: Vec<CardanoCertKey>,
        origin: Option<String>,
    ) -> Self {
        Self {
            request_id,
            sign_data,
            utxos,
            cert_keys,
            origin,
        }
    }

    fn get_map_size(&self) -> u64 {
        let mut size = 3;
        if let Some(_) = self.request_id {
            size = size + 1;
        }
        if let Some(_) = self.origin {
            size = size + 1;
        }
        size
    }
}

impl RegistryItem for CardanoSignRequest {
    fn get_registry_type() -> RegistryType<'static> {
        CARDANO_SIGN_REQUEST
    }
}

impl<C> minicbor::Encode<C> for CardanoSignRequest {
    fn encode<W: Write>(&self, e: &mut Encoder<W>, _ctx: &mut C) -> Result<(), Error<W::Error>> {
        e.map(self.get_map_size())?;

        if let Some(request_id) = &self.request_id {
            e.int(Int::from(REQUEST_ID))?
                .tag(Tag::Unassigned(UUID.get_tag()))?
                .bytes(request_id)?;
        }

        e.int(Int::from(SIGN_DATA))?.bytes(&self.sign_data)?;

        e.int(Int::from(UTXOS))?.array(self.utxos.len() as u64)?;
        for x in &self.utxos {
            e.tag(Tag::Unassigned(CARDANO_UTXO.get_tag()))?;
            x.encode(e, _ctx)?;
        }

        e.int(Int::from(CERT_KEYS))?
            .array(self.cert_keys.len() as u64)?;
        for cert_key in &self.cert_keys {
            e.tag(Tag::Unassigned(CARDANO_CERT_KEY.get_tag()))?;
            cert_key.encode(e, _ctx)?;
        }

        if let Some(origin) = &self.origin {
            e.int(Int::from(ORIGIN))?.str(origin)?;
        }

        Ok(())
    }
}

impl<'b, C> minicbor::Decode<'b, C> for CardanoSignRequest {
    fn decode(d: &mut Decoder<'b>, _ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let mut cardano_sign_request = CardanoSignRequest::default();
        cbor_map(d, &mut cardano_sign_request, |key, obj, d| {
            let key =
                u8::try_from(key).map_err(|e| minicbor::decode::Error::message(e.to_string()))?;
            match key {
                REQUEST_ID => {
                    d.tag()?;
                    obj.set_request_id(Some(d.bytes()?.to_vec()));
                }
                SIGN_DATA => {
                    obj.set_sign_data(d.bytes()?.to_vec());
                }
                UTXOS => {
                    cbor_array(d, &mut obj.utxos, |index, array, d| {
                        d.tag()?;
                        array.push(CardanoUTXO::decode(d, _ctx)?);
                        Ok(())
                    })?;
                }
                CERT_KEYS => {
                    cbor_array(d, &mut obj.cert_keys, |index, array, d| {
                        d.tag()?;
                        array.push(CardanoCertKey::decode(d, _ctx)?);
                        Ok(())
                    })?;
                }
                ORIGIN => obj.set_origin(Some(d.str()?.to_string())),
                _ => {}
            }
            Ok(())
        })?;
        Ok(cardano_sign_request)
    }
}

impl To for CardanoSignRequest {
    fn to_bytes(&self) -> URResult<Vec<u8>> {
        minicbor::to_vec(self.clone()).map_err(|e| URError::CborDecodeError(e.to_string()))
    }
}

impl FromCbor<CardanoSignRequest> for CardanoSignRequest {
    fn from_cbor(bytes: Vec<u8>) -> URResult<CardanoSignRequest> {
        minicbor::decode(&bytes).map_err(|e| URError::CborDecodeError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_key_path::PathComponent;
    use alloc::vec;

    extern crate std;

    use std::println;

    #[test]
    fn test_construct() {
        let sign_data = hex::decode("84a400828258204e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c99038258204e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c99040182a200581d6179df4c75f7616d7d1fd39cbc1a6ea6b40a0d7b89fea62fc0909b6c370119c350a200581d61c9b0c9761fd1dc0404abd55efc895026628b5035ac623c614fbad0310119c35002198ecb0300a0f5f6").unwrap();
        let signing_key_1 = CryptoKeyPath::new(
            vec![
                PathComponent::new(Some(1852), true).unwrap(),
                PathComponent::new(Some(1815), true).unwrap(),
                PathComponent::new(Some(0), true).unwrap(),
                PathComponent::new(Some(0), false).unwrap(),
                PathComponent::new(Some(0), false).unwrap(),
            ],
            Some([0x73, 0xc5, 0xda, 0x0a]),
            None,
        );
        let signing_key_2 = CryptoKeyPath::new(
            vec![
                PathComponent::new(Some(1852), true).unwrap(),
                PathComponent::new(Some(1815), true).unwrap(),
                PathComponent::new(Some(0), true).unwrap(),
                PathComponent::new(Some(0), false).unwrap(),
                PathComponent::new(Some(1), false).unwrap(),
            ],
            Some([0x73, 0xc5, 0xda, 0x0a]),
            None,
        );
        let utxos = vec![
            CardanoUTXO::new(
                hex::decode("4e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c99").unwrap(),
                3,
                10000000,
                signing_key_1,
                "addr1qy8ac7qqy0vtulyl7wntmsxc6wex80gvcyjy33qffrhm7sh927ysx5sftuw0dlft05dz3c7revpf7jx0xnlcjz3g69mq4afdhv".to_string(),
            ),
            CardanoUTXO::new(
                hex::decode("4e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c99").unwrap(),
                4,
                18020000,
                signing_key_2,
                "addr1qyz85693g4fr8c55mfyxhae8j2u04pydxrgqr73vmwpx3azv4dgkyrgylj5yl2m0jlpdpeswyyzjs0vhwvnl6xg9f7ssrxkz90".to_string(),
            ),
        ];

        let cert_key_path = CryptoKeyPath::new(
            vec![
                PathComponent::new(Some(1852), true).unwrap(),
                PathComponent::new(Some(1815), true).unwrap(),
                PathComponent::new(Some(0), true).unwrap(),
                PathComponent::new(Some(2), false).unwrap(),
                PathComponent::new(Some(0), false).unwrap(),
            ],
            Some([0x73, 0xc5, 0xda, 0x0a]),
            None,
        );
        let cert_keys = vec![CardanoCertKey::new(
            hex::decode("e557890352095f1cf6fd2b7d1a28e3c3cb029f48cf34ff890a28d176").unwrap(),
            cert_key_path,
        )];

        let request_id = Some(
            [
                155, 29, 235, 77, 59, 125, 75, 173, 155, 221, 43, 13, 123, 61, 203, 109,
            ]
            .to_vec(),
        );

        let cardano_sign_request = CardanoSignRequest::new(
            request_id,
            sign_data,
            utxos,
            cert_keys,
            Some("cardano-wallet".to_string()),
        );

        assert_eq!("a501d825509b1deb4d3b7d4bad9bdd2b0d7b3dcb6d0258a184a400828258204e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c99038258204e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c99040182a200581d6179df4c75f7616d7d1fd39cbc1a6ea6b40a0d7b89fea62fc0909b6c370119c350a200581d61c9b0c9761fd1dc0404abd55efc895026628b5035ac623c614fbad0310119c35002198ecb0300a0f5f60382d90899a50158204e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c990203031a0098968004d90130a2018a19073cf5190717f500f500f400f4021a73c5da0a0578676164647231717938616337717179307674756c796c37776e746d737863367765783830677663796a79333371666672686d37736839323779737835736674757730646c66743035647a3363377265767066376a7830786e6c636a7a336736396d71346166646876d90899a50158204e3a6e7fdcb0d0efa17bf79c13aed2b4cb9baf37fb1aa2e39553d5bd720c5c990204031a0112f6a004d90130a2018a19073cf5190717f500f500f401f4021a73c5da0a057867616464723171797a383536393367346672386335356d667978686165386a3275303470796478726771723733766d77707833617a763464676b797267796c6a35796c326d306a6c70647065737779797a6a7330766877766e6c367867396637737372786b7a39300481d9089ca201581ce557890352095f1cf6fd2b7d1a28e3c3cb029f48cf34ff890a28d17602d90130a2018a19073cf5190717f500f502f400f4021a73c5da0a056e63617264616e6f2d77616c6c6574", hex::encode(cardano_sign_request.to_bytes().unwrap()))
    }
}