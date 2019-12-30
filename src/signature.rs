use ring::{
    rand::SystemRandom,
    signature::{
        self,
        KeyPair,
        Ed25519KeyPair,
        UnparsedPublicKey,
    },
};

#[derive(Debug)]
pub enum MyError {
    BadSignature,
}

fn generate_bytes() -> Vec<u8> {
    let rng = SystemRandom::new();
    Ed25519KeyPair::generate_pkcs8(&rng).unwrap().as_ref().to_vec()
}

pub fn new_key() -> Ed25519KeyPair {
    Ed25519KeyPair::from_pkcs8(&generate_bytes()).unwrap()
}

pub fn load_key(bytes: &[u8]) -> Ed25519KeyPair {
    Ed25519KeyPair::from_pkcs8(bytes).unwrap()
}

pub fn sign(key_pair: &Ed25519KeyPair, message: &[u8]) -> Vec<u8> {
    key_pair.sign(message).as_ref().iter().cloned().collect()
}

pub fn verify(key_pair: &Ed25519KeyPair, msg: &[u8], sig: &[u8]) -> Result<(), MyError> {
    let peer_public_key = UnparsedPublicKey::new(&signature::ED25519, key_pair.public_key().as_ref());
    peer_public_key.verify(msg, sig.as_ref())
        .map_err(|_| MyError::BadSignature)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        let msg = "test12";
        let key = new_key();
        let signature: &[u8] = &sign(&key, msg.as_bytes());
        let result = verify(&key, msg.as_bytes(), signature).unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn test_verify_error() {
        let msg = "test12";
        let msg2 = "test34";
        let key = new_key();
        match verify(&key, msg2.as_bytes(), &sign(&key, msg.as_bytes())) {
            Ok(_) => assert!(false, "db file should not exist"),
            Err(MyError::BadSignature) => assert!(true),
        };
    }
}
