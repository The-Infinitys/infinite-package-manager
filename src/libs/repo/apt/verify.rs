use crate::modules::error::UpmError;
use sequoia_openpgp::{
    Cert, KeyHandle,
    anyhow::{self, Result},
    cert::CertParser,
    parse::{
        PacketParser, Parse,
        stream::{MessageLayer, MessageStructure, VerificationHelper, VerifierBuilder},
    },
    policy::StandardPolicy,
};
use std::io::Read;

// 1. 構造体を Option<Vec<Cert>> に戻す
struct Helper {
    certs: Vec<Cert>,
}

impl Helper {
    pub fn new(signature: Option<impl AsRef<[u8]>>) -> Result<Self, UpmError> {
        let mut certs_vec = Vec::new();
        if let Some(signature) = signature {
            let signature = signature.as_ref();
            let ppr = PacketParser::from_bytes(signature)?;
            for cert_result in CertParser::from(ppr) {
                match cert_result {
                    Ok(cert) => {
                        certs_vec.push(cert);
                    }
                    Err(err) => {
                        // 不正な Cert が含まれていても、ログを出力して続行
                        eprintln!("Error reading keyring: {}", err);
                    }
                }
            }
        }
        Ok(Self { certs: certs_vec })
    }

    pub fn lookup_cert_by_handle(
        &self,
        id: &KeyHandle,
    ) -> Result<Cert, sequoia_openpgp::anyhow::Error> {
        if let Some(cert) = self.certs.iter().find(|cert| cert.key_handle() == *id) {
            return Ok(cert.clone());
        }
        Err(sequoia_openpgp::anyhow::Error::msg(
            "Public key not found or KeyHandle mismatch",
        ))
    }
}

impl VerificationHelper for Helper {
    fn get_certs(&mut self, ids: &[KeyHandle]) -> Result<Vec<Cert>> {
        if self.certs.is_empty() {
            return Ok(Vec::new());
        }

        let mut found_certs = Vec::new();
        for id in ids {
            found_certs.push(self.lookup_cert_by_handle(id)?);
        }
        Ok(found_certs)
    }

    fn check(&mut self, structure: MessageStructure) -> Result<()> {
        let mut signature_found = false;
        let mut has_valid_signature = false;

        for layer in structure.into_iter() {
            match layer {
                MessageLayer::Encryption { .. } | MessageLayer::Compression { .. } => {}

                MessageLayer::SignatureGroup { ref results } => {
                    signature_found = true;
                    if results.iter().any(|r| r.is_ok()) {
                        has_valid_signature = true;
                    }
                }
            }
        }

        // 公開鍵が指定されている（certsが空でない）場合のみ署名検証を行う
        if !self.certs.is_empty() {
            if !signature_found {
                return Err(anyhow::anyhow!(
                    "Message must contain a signature when public key is provided"
                ));
            }
            if !has_valid_signature {
                return Err(anyhow::anyhow!("No valid signature found"));
            }
        }

        Ok(())
    }
}
pub fn verification(
    signed_context: impl AsRef<[u8]>,
    public_gpg: Option<impl AsRef<[u8]>>,
) -> Result<Vec<u8>, UpmError> {
    let signed_context = signed_context.as_ref();
    let p = &StandardPolicy::new();
    let h = Helper::new(public_gpg)?;
    let mut v = VerifierBuilder::from_bytes(signed_context)?.with_policy(p, None, h)?;
    let mut content = Vec::new();
    v.read_to_end(&mut content)?;
    Ok(content)
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_verifier() -> Result<(), UpmError> {
        let signed_message = include_str!("../../../../tests/verify/original_text.txt.asc");
        let original_message = include_str!("../../../../tests/verify/original_text.txt");
        let public_gpg = include_bytes!("../../../../tests/verify/public.gpg");
        let invalid_keyhandle_gpg = include_bytes!("../../../../tests/verify/invalid_public.gpg");
        let incorrect_gpg = include_bytes!("../../../../tests/verify/public_incorrect.gpg");

        let result = verification(signed_message, Some(public_gpg))?;
        let result = String::from_utf8(result)?;
        assert_eq!(&result, original_message);

        let result = verification(signed_message, Some(invalid_keyhandle_gpg));
        assert!(result.is_err());

        let result = verification(signed_message, Some(incorrect_gpg));
        assert!(result.is_err());

        // public_gpg が None の場合に成功することを確認
        let result = verification(signed_message, None::<Vec<u8>>);
        assert!(result.is_ok());

        Ok(())
    }
}
