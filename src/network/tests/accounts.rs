use crate::{
    config::Profile,
    network::{Account, Role},
    Data, Event,
};

use rand::prelude::*;
use rsa::{RsaPrivateKey, RsaPublicKey};

#[test]
fn test_account_sign_event() {
    // Create already signed event
    let mut rng = rand::thread_rng();
    let (pub_key, priv_key): (RsaPublicKey, RsaPrivateKey) = {
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        (pub_key, priv_key)
    };
    let id = rng.gen();

    let user: Account = Account::new(
        Role::User,
        Profile::new(None, None, None, None, None),
        pub_key,
        priv_key,
        id,
    );
    let event: Event<Data> = user.new_event(Data::GroupMessage(String::from("Hello")));
    assert!(event.verify_sign(&user.pub_key));
}

#[test]
fn test_encrypt_decrypt() {
    let mut rng = rand::thread_rng();
    let (pub_key, priv_key): (RsaPublicKey, RsaPrivateKey) = {
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        (pub_key, priv_key)
    };

    let id = rng.gen();

    let user: Account = Account::new(
        Role::User,
        Profile::new(None, None, None, None, None),
        pub_key,
        priv_key,
        id,
    );

    let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let enc_data = user.encrypt_msg(&data, &user.pub_key);
    let dec_data = user.decrypt_msg(&enc_data);

    assert_eq!(data, dec_data);
}
