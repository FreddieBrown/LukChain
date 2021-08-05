use crate::blockchain::{
    events::{Data, Event},
    Block, BlockChain,
};
use crate::config::Profile;
use crate::network::{Account, Role};

// Basic tests
#[test]
fn create_blockchain() {
    let bc: BlockChain = BlockChain::new();
    assert!(bc.chain.len() == 0);
    assert!(bc.users.len() == 0);
}

#[test]
fn create_event() {
    let event: Event = Event::new(10, Data::GroupMessage(String::from("Hello")));
    assert_eq!(event.made_by, 10);
    match event.data {
        Data::GroupMessage(m) => assert_eq!(m, String::from("Hello")),
        _ => unreachable!(),
    };
}

#[test]
fn add_event_to_block() {
    let mut block: Block = Block::new(None);
    let event: Event = Event::new(10, Data::GroupMessage(String::from("Hello")));
    block.add_event(event);
    assert!(block.get_event_count() == 1);
}

#[test]
fn add_block_to_blockchain() {
    let mut bc: BlockChain = BlockChain::new();
    let mut block: Block = Block::new(None);
    let event: Event = Event::new(10, Data::GroupMessage(String::from("Hello")));
    block.add_event(event);
    assert!(block.get_event_count() == 1);
    assert!(bc.append(block).is_ok());
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
        },
    );
    let mut event: Event = Event::new(user.id, Data::GroupMessage(String::from("Hello")));
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
        },
    );
    let user1: Account = Account::new(
        Role::User,
        Profile {
            pub_key: None,
            priv_key: None,
            block_size: None,
        },
    );
    let mut event: Event = Event::new(user.id, Data::GroupMessage(String::from("Hello")));
    user1.sign_event(&mut event);
    assert!(!event.verify_sign(&user.pub_key));
}
