use crate::blockchain::{
    config::Profile,
    events::{Data, Event},
    network::{Account, Role},
    Block, BlockChain, UserPair,
};

use std::sync::Arc;

// Basic tests
#[test]
fn create_blockchain() {
    let bc: BlockChain<Data> = BlockChain::new();
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
    let mut bc: BlockChain<Data> = BlockChain::new();
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
    let user: Account = Account::new(
        Role::User,
        Profile {
            pub_key: None,
            priv_key: None,
            block_size: None,
            lookup_address: None,
            lookup_filter: None,
        },
    );
    let mut event: Event<Data> = Event::new(user.id, Data::GroupMessage(String::from("Hello")));
    user.sign_event(&mut event);
    assert!(event.verify_sign(&user.pub_key));
}

#[test]
fn sign_message_with_wrong_key() {
    let user: Account = Account::new(
        Role::User,
        Profile {
            pub_key: None,
            priv_key: None,
            block_size: None,
            lookup_address: None,
            lookup_filter: None,
        },
    );
    let user1: Account = Account::new(
        Role::User,
        Profile {
            pub_key: None,
            priv_key: None,
            block_size: None,
            lookup_address: None,
            lookup_filter: None,
        },
    );
    let mut event: Event<Data> = Event::new(user.id, Data::GroupMessage(String::from("Hello")));
    user1.sign_event(&mut event);
    assert!(!event.verify_sign(&user.pub_key));
}
