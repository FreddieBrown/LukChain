use crate::blockchain::network::{participants::shared::*, ConnectionPool, Role};
use crate::blockchain::{Block, BlockChain, Data, Event, Profile, UserPair};

use std::net::SocketAddr;
use std::sync::Arc;

// Test `setup
#[tokio::test]
async fn test_open_specific_port() {
    let port = 6666;
    let listener = setup(Some(port)).await;
    match listener {
        Ok(l) => {
            let addr: SocketAddr = l.local_addr().unwrap();
            assert_eq!(addr.port(), port);
        }
        _ => assert!(false),
    };
}

#[tokio::test]
async fn test_open_any_port() {
    let listener = setup(None).await;
    match listener {
        Ok(l) => {
            let addr: SocketAddr = l.local_addr().unwrap();
            assert!(addr.port() > 0);
        }
        _ => assert!(false),
    };
}

// Test `clear_connection_pool`
#[tokio::test]
async fn test_clear_cp_with_empty_buffer() {
    let pair: Arc<UserPair<Data>> = Arc::new(
        UserPair::new(
            Role::User,
            Profile::new(None, None, None, None, None),
            false,
        )
        .await
        .unwrap(),
    );
    let cp: Arc<ConnectionPool> = Arc::new(ConnectionPool::new());
    assert!(clear_connection_pool(pair, cp).await.is_ok());
}

// Test `replace_blockchain`
#[tokio::test]
async fn test_replace_blockchain_works() {
    let pair: Arc<UserPair<Data>> = Arc::new(
        UserPair::new(
            Role::User,
            Profile::new(None, None, None, None, None),
            false,
        )
        .await
        .unwrap(),
    );

    let new_bc: BlockChain<Data> = BlockChain::new(None);

    assert!(replace_blockchain(Arc::clone(&pair), &new_bc).await.is_ok());

    let unlocked_bc = pair.node.blockchain.write().await;

    assert_eq!(new_bc, *unlocked_bc);
}

// Test `add_block`
#[tokio::test]
async fn test_add_block_works() {
    let pair: Arc<UserPair<Data>> = Arc::new(
        UserPair::new(
            Role::User,
            Profile::new(None, None, None, None, None),
            false,
        )
        .await
        .unwrap(),
    );

    let mut block: Block<Data> = Block::new(pair.node.last_hash().await);
    let event: Event<Data> = Event::new(
        pair.node.account.id,
        Data::GroupMessage(String::from("Hello")),
    );
    block.add_event(event.clone());
    // add event to loose events
    pair.node.add_loose_event(event.clone()).await;

    assert!(add_block(Arc::clone(&pair), &block).await.is_ok());
}

#[tokio::test]
async fn test_events_in_loose_events_removed() {
    let pair: Arc<UserPair<Data>> = Arc::new(
        UserPair::new(
            Role::User,
            Profile::new(None, None, None, None, None),
            false,
        )
        .await
        .unwrap(),
    );
    let mut block: Block<Data> = Block::new(pair.node.last_hash().await);
    let event: Event<Data> = Event::new(
        pair.node.account.id,
        Data::GroupMessage(String::from("Hello")),
    );
    block.add_event(event.clone());

    assert!(add_block(Arc::clone(&pair), &block).await.is_ok());
}

#[tokio::test]
async fn test_events_left_in_loose_events() {
    let pair: Arc<UserPair<Data>> = Arc::new(
        UserPair::new(
            Role::User,
            Profile::new(None, None, None, None, None),
            false,
        )
        .await
        .unwrap(),
    );
    let mut block: Block<Data> = Block::new(pair.node.last_hash().await);
    let event1: Event<Data> = Event::new(
        pair.node.account.id,
        Data::GroupMessage(String::from("Hello1")),
    );
    let event2: Event<Data> = Event::new(
        pair.node.account.id,
        Data::GroupMessage(String::from("Hello2")),
    );
    block.add_event(event1.clone());
    // add events to loose events
    pair.node.add_loose_event(event1.clone()).await;
    pair.node.add_loose_event(event2.clone()).await;

    assert!(add_block(Arc::clone(&pair), &block).await.is_ok());

    // Check event 2 still in loose events
    let loose_events = pair.node.loose_events.read().await;
    assert_eq!(loose_events.len(), 1);
    assert_eq!(loose_events[0], event2);
}
