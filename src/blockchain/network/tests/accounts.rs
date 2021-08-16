use crate::blockchain::{
    config::Profile,
    network::{Account, Role},
    Data, Event,
};

use rand::prelude::*;
use rsa::{RsaPrivateKey, RsaPublicKey};

#[test]
fn account_sign_event() {
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
        Profile {
            block_size: None,
            lookup_address: None,
            lookup_filter: None,
            user_data: None,
        },
        pub_key,
        priv_key,
        id,
    );
    let event: Event<Data> = user.new_event(Data::GroupMessage(String::from("Hello")));
    assert!(event.verify_sign(&user.pub_key));
}
