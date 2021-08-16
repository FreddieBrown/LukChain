//! Functions to run tests on participants in the network

use crate::blockchain::{
    config::Profile,
    network::{
        accounts::{Account, Role},
        lookup_run,
        messages::{traits::ReadLengthPrefix, MessageData, NetworkMessage},
        participants_run, send_message,
    },
    Data, UserPair,
};

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use futures::future::{AbortHandle, Abortable};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_sanity() {
    assert_eq!(1, 1);
}

#[tokio::test]
#[traced_test]
async fn test_lookup_and_connect() {
    let (lookup_handle, lookup_registration) = AbortHandle::new_pair();
    let (part_handle, part_registration) = AbortHandle::new_pair();
    let notify: Arc<Notify> = Arc::new(Notify::new());

    // Lookup
    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8281)).await.unwrap();
        }),
        lookup_registration,
    );

    // Participant
    let noti_cpy = Arc::clone(&notify);
    let _future2 = Abortable::new(
        tokio::spawn(async move {
            // Define profile
            let profile =
                Profile::new(None, None, None, Some(String::from("127.0.0.1:8281")), None);

            noti_cpy.notified().await;
            let pair: Arc<UserPair<Data>> =
                Arc::new(UserPair::new(Role::User, profile, false).await?);
            participants_run::<Data>(pair, None).await
        }),
        part_registration,
    );

    // Participant Tester
    tokio::spawn(async move {
        // Connect to the LookUp
        let mut buffer = [0_u8; 4096];
        let lookup_addr = "127.0.0.1:8281";
        let profile = Profile::new(None, None, None, Some(String::from("127.0.0.1:8281")), None);
        let account: Account = Account::new(Role::User, profile);

        // Setup Network connect
        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();

        let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)))
            .await
            .unwrap();

        let inbound_addr = listener.local_addr().unwrap();

        println!("TEST ADDR: {:?}", inbound_addr.to_string());

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(
            1,
            inbound_addr.to_string(),
            account.role,
        ));

        send_message(&mut stream, reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();

        assert!(matches!(recv_message.data, MessageData::Confirm));

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, finish_message).await.unwrap();

        notify.notify_one();
        // StartUp Listener and wait for partcipant to connect
        if let Ok((mut inbound, _)) = listener.accept().await {
            println!("CONNECTION");
            // Test intro connection protocol
            // Accept initial ID Message
            let mut buffer = [0_u8; 4096];
            match NetworkMessage::from_stream(&mut inbound, &mut buffer).await {
                Ok(m) => assert!(matches!(m.data, MessageData::<Data>::InitialID(_, _, _))),
                _ => assert!(false),
            };
        }
    })
    .await
    .unwrap();

    lookup_handle.abort();
    part_handle.abort();
}
