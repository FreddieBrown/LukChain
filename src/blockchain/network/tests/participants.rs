//! Functions to run tests on participants in the network

use crate::blockchain::{
    config::Profile,
    network::{
        accounts::{Account, Role},
        lookup_run,
        messages::{traits::ReadLengthPrefix, MessageData, NetworkMessage},
        participants_run, send_message,
    },
    Data,
};

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use futures::future::{AbortHandle, Abortable};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
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

    // Lookup
    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8281)).await.unwrap();
        }),
        lookup_registration,
    );

    // Participant
    let _future2 = Abortable::new(
        tokio::spawn(async move {
            // Define profile
            let profile = Profile::new(None, None, None, Some(String::from("127.0.0.1:8281")));

            sleep(Duration::from_millis(100)).await;
            participants_run::<Data>(profile, None, Role::User, None).await
        }),
        part_registration,
    );

    // Participant Tester
    tokio::spawn(async move {
        // Connect to the LookUp
        let mut buffer = [0_u8; 4096];
        let lookup_addr = "127.0.0.1:8281";
        let profile = Profile::new(None, None, None, Some(String::from("127.0.0.1:8281")));
        let account: Account = Account::new(Role::User, profile);

        // Setup Network connect
        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();

        let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)))
            .await
            .unwrap();

        let inbound_addr = listener.local_addr().unwrap();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(
            1,
            inbound_addr.to_string(),
            account.role,
        ));

        send_message::<Data>(&mut stream, reg_message)
            .await
            .unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();

        assert!(matches!(recv_message.data, MessageData::Confirm));

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, finish_message).await.unwrap();

        // StartUp Listener and wait for partcipant to connect
        if let Ok((mut inbound, _)) = listener.accept().await {
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
