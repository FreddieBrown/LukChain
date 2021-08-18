use crate::{
    config::Profile,
    events::{Data, Event},
    network::{Account, Role},
    Block, BlockChain, UserPair,
};

use rand::prelude::*;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::sync::Arc;

// Basic tests
#[test]
fn create_blockchain() {
    let bc: BlockChain<Data> = BlockChain::new(None);
    assert!(bc.chain.len() == 0);
    assert!(bc.users.len() == 0);
}

#[test]
fn create_event() {
    let event: Event<Data> = Event::new(10, Data::GroupMessage(String::from("Hello")));
    assert_eq!(event.made_by, 10);
    match event.data {
        Data::GroupMessage(m) => assert_eq!(m, String::from("Hello")),
        _ => unreachable!(),
    };
}

#[test]
fn add_event_to_block() {
    let mut block: Block<Data> = Block::new(None);
    let event: Event<Data> = Event::new(10, Data::GroupMessage(String::from("Hello")));
    block.add_event(event);
    assert!(block.get_event_count() == 1);
}

#[tokio::test]
async fn add_block_to_blockchain() {
    let mut bc: BlockChain<Data> = BlockChain::new(None);
    let mut block: Block<Data> = Block::new(None);
    let event: Event<Data> = Event::new(10, Data::GroupMessage(String::from("Hello")));
    let pair: Arc<UserPair<Data>> = Arc::new(
        UserPair::new(
            Role::User,
            Profile::new(None, None, None, None, None),
            false,
        )
        .await
        .unwrap(),
    );
    block.add_event(event);
    assert!(block.get_event_count() == 1);
    assert!(bc.append(block, pair).await.is_ok());
    assert!(bc.chain.len() == 1);
}

// Crypto tests
#[test]
fn sign_event_with_key() {
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
    let mut event: Event<Data> = Event::new(user.id, Data::GroupMessage(String::from("Hello")));
    user.sign_event(&mut event);
    assert!(event.verify_sign(&user.pub_key));
}

#[test]
fn sign_message_with_wrong_key() {
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

    let (pub_key2, priv_key2): (RsaPublicKey, RsaPrivateKey) = {
        let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        (pub_key, priv_key)
    };
    let id2 = rng.gen();

    let user1: Account = Account::new(
        Role::User,
        Profile::new(None, None, None, None, None),
        pub_key2,
        priv_key2,
        id2,
    );
    let mut event: Event<Data> = Event::new(user.id, Data::GroupMessage(String::from("Hello")));
    user1.sign_event(&mut event);
    assert!(!event.verify_sign(&user.pub_key));
}
