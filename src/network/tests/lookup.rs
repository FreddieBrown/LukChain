use crate::{
    network::{
        accounts::Role,
        lookup::lookup_run,
        messages::{traits::ReadLengthPrefix, MessageData, NetworkMessage},
        send_message,
    },
    Data,
};

use futures::future::{AbortHandle, Abortable};
use tokio::net::TcpStream;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_sanity() {
    assert_eq!(1, 1);
}

#[tokio::test]
#[traced_test]
async fn test_lookup_registration() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8181)).await.unwrap();
        }),
        abort_registration,
    );
    tokio::spawn(async move {
        // Connect to the LookUp
        let mut buffer = [0_u8; 4096];
        let lookup_addr = "127.0.0.1:8181";
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(1, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_request_less_than_4() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8182)).await.unwrap();
        }),
        abort_registration,
    );
    tokio::spawn(async move {
        // Connect to the LookUp
        connect_test(1, String::from("127.0.0.1:8182"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        let id = 2;
        let lookup_addr = "127.0.0.1:8182";
        let mut buffer = [0_u8; 4096];
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));

        // Do random 4 lookup
        let lookup_msg = NetworkMessage::<Data>::new(MessageData::GeneralAddrRequest(id, None));
        send_message(&mut stream, &lookup_msg).await.unwrap();

        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();

        // Assert there is 1 address, which is not the address of process
        match recv_message.data {
            MessageData::PeerAddresses(v) => {
                assert_eq!(v.len(), 1);
                assert!(!v.contains(&(id, stream.local_addr().unwrap().to_string())));
            }
            _ => assert!(false),
        };

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_request_4() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8183)).await.unwrap();
        }),
        abort_registration,
    );

    // Register Nodes
    tokio::spawn(async move {
        connect_test(1, String::from("127.0.0.1:8183"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        connect_test(2, String::from("127.0.0.1:8183"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        // Connect to the LookUp
        connect_test(3, String::from("127.0.0.1:8183"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        connect_test(4, String::from("127.0.0.1:8183"), Role::User).await;
    })
    .await
    .unwrap();

    tokio::spawn(async move {
        // Register details with LookUp node
        let id = 5;
        let lookup_addr = "127.0.0.1:8183";
        let mut buffer = [0_u8; 4096];
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));
        // Do random 4 lookup
        let lookup_msg = NetworkMessage::<Data>::new(MessageData::GeneralAddrRequest(id, None));
        send_message(&mut stream, &lookup_msg).await.unwrap();

        // Assert there are 4 addresses
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();

        // Assert there is 1 address, which is not the address of process
        match recv_message.data {
            MessageData::PeerAddresses(v) => {
                assert_eq!(v.len(), 4);
                assert!(!v.contains(&(id, stream.local_addr().unwrap().to_string())));
            }
            _ => assert!(false),
        };
        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_when_empty() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8184)).await.unwrap();
        }),
        abort_registration,
    );
    tokio::spawn(async move {
        // Register details with LookUp node
        let id = 1;
        let lookup_addr = "127.0.0.1:8184";
        let mut buffer = [0_u8; 4096];
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));
        // Do random 4 lookup
        let lookup_msg = NetworkMessage::<Data>::new(MessageData::GeneralAddrRequest(id, None));
        send_message(&mut stream, &lookup_msg).await.unwrap();

        // Assert there are 4 addresses
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        // Assert return is NoAddr
        assert!(matches!(recv_message.data, MessageData::NoAddr));
        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_request_1() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8185)).await.unwrap();
        }),
        abort_registration,
    );
    tokio::spawn(async move {
        // Register details with LookUp node
        connect_test(1, String::from("127.0.0.1:8185"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        let id = 2;
        let lookup_addr = "127.0.0.1:8185";
        let mut buffer = [0_u8; 4096];
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));
        // Do random 4 lookup
        let lookup_msg = NetworkMessage::<Data>::new(MessageData::RequestAddress(1));
        send_message(&mut stream, &lookup_msg).await.unwrap();

        // Assert there are 4 addresses
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        // Assert return is NoAddr
        assert!(matches!(recv_message.data, MessageData::PeerAddress(_)));

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_request_4_noone_in_role() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8186)).await.unwrap();
        }),
        abort_registration,
    );

    // Register Nodes
    tokio::spawn(async move {
        connect_test(1, String::from("127.0.0.1:8186"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        connect_test(2, String::from("127.0.0.1:8186"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        // Connect to the LookUp
        connect_test(3, String::from("127.0.0.1:8186"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        connect_test(4, String::from("127.0.0.1:8186"), Role::User).await;
    })
    .await
    .unwrap();

    tokio::spawn(async move {
        // Register details with LookUp node
        let id = 5;
        let lookup_addr = "127.0.0.1:8186";
        let mut buffer = [0_u8; 4096];
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));
        // Do random 4 lookup
        let lookup_msg =
            NetworkMessage::<Data>::new(MessageData::GeneralAddrRequest(id, Some(Role::Miner)));
        send_message(&mut stream, &lookup_msg).await.unwrap();

        // Assert there are 4 addresses
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();

        // Assert there is no addresses
        assert!(matches!(recv_message.data, MessageData::NoAddr));

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_request_4_less_in_role() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8187)).await.unwrap();
        }),
        abort_registration,
    );

    // Register Nodes
    tokio::spawn(async move {
        connect_test(1, String::from("127.0.0.1:8187"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        connect_test(2, String::from("127.0.0.1:8187"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        // Connect to the LookUp
        connect_test(3, String::from("127.0.0.1:8187"), Role::User).await;
    })
    .await
    .unwrap();
    tokio::spawn(async move {
        connect_test(4, String::from("127.0.0.1:8187"), Role::Miner).await;
    })
    .await
    .unwrap();

    tokio::spawn(async move {
        // Register details with LookUp node
        let id = 5;
        let lookup_addr = "127.0.0.1:8187";
        let mut buffer = [0_u8; 4096];
        let role = Role::User;

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
        let own_addr = stream.local_addr().unwrap().to_string();

        // Register details
        let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

        send_message(&mut stream, &reg_message).await.unwrap();

        // If get back correct message, end connection
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();
        assert!(matches!(recv_message.data, MessageData::Confirm));
        // Do random 4 lookup
        let lookup_msg =
            NetworkMessage::<Data>::new(MessageData::GeneralAddrRequest(id, Some(Role::Miner)));
        send_message(&mut stream, &lookup_msg).await.unwrap();

        // Assert there are 4 addresses
        let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
            .await
            .unwrap();

        println!("{:?}", recv_message);

        match recv_message.data {
            MessageData::PeerAddresses(v) => {
                assert_eq!(v.len(), 1);
                assert!(!v.contains(&(id, stream.local_addr().unwrap().to_string())));
            }
            _ => assert!(false),
        };

        let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
        send_message(&mut stream, &finish_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

#[tokio::test]
#[traced_test]
async fn test_lookup_strike() {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    let _future = Abortable::new(
        tokio::spawn(async move {
            // Startup LookUp Node
            lookup_run::<Data>(Some(8188)).await.unwrap();
        }),
        abort_registration,
    );

    // Register Nodes
    tokio::spawn(async move {
        connect_test(1, String::from("127.0.0.1:8188"), Role::User).await;
    })
    .await
    .unwrap();

    tokio::spawn(async move {
        // Register details with LookUp node
        let lookup_addr = "127.0.0.1:8188";

        let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();

        // Send a strike message
        let strike_message = NetworkMessage::<Data>::new(MessageData::Strike(1));
        send_message(&mut stream, &strike_message).await.unwrap();
    })
    .await
    .unwrap();

    abort_handle.abort();
}

async fn connect_test(id: u128, lookup_addr: String, role: Role) {
    // Connect to the LookUp
    let mut buffer = [0_u8; 4096];

    let mut stream: TcpStream = TcpStream::connect(lookup_addr).await.unwrap();
    let own_addr = stream.local_addr().unwrap().to_string();

    // Register details
    let reg_message = NetworkMessage::<Data>::new(MessageData::LookUpReg(id, own_addr, role));

    send_message(&mut stream, &reg_message).await.unwrap();

    // If get back correct message, end connection
    let recv_message = NetworkMessage::<Data>::from_stream(&mut stream, &mut buffer)
        .await
        .unwrap();
    assert!(matches!(recv_message.data, MessageData::Confirm));

    let finish_message = NetworkMessage::<Data>::new(MessageData::Finish);
    send_message(&mut stream, &finish_message).await.unwrap();
}
