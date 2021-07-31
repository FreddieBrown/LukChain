use blockchat::{
    accounts::{Account, Role},
    blockchain::{
        events::{Data, Event},
        Block, BlockChain,
    },
};

fn main() {
    // Creat users
    let user1 = Account::new(Role::User);
    let user2 = Account::new(Role::User);

    // Create blockchain
    let mut bc: BlockChain = BlockChain::new();

    // register users
    bc.new_user(user1.id, user1.pub_key.clone());
    bc.new_user(user2.id, user2.pub_key.clone());

    // Create starting block of chain
    let mut genesis: Block = Block::new(None);

    // create 2 events
    let event1: Event = user1.new_event(Data::new_individual_message(
        user2.id,
        &user2.pub_key,
        String::from("Message1"),
    ));
    let event2: Event = user1.new_event(Data::GroupMessage(String::from("Message2")));

    // add events to block
    genesis.add_event(event1);
    genesis.add_event(event2);

    // add event to blockchain
    bc.append(genesis).unwrap();

    // create second block
    let mut second: Block = Block::new(bc.last_hash());

    // more events
    let event3: Event = user2.new_event(Data::new_individual_message(
        user1.id,
        &user1.pub_key,
        String::from("Message3"),
    ));
    let event4: Event = user2.new_event(Data::GroupMessage(String::from("Message4")));
    let event5: Event = user2.new_event(Data::NewUser {
        id: user2.id,
        pub_key: user2.pub_key.clone(),
    });

    second.add_event(event3);
    second.add_event(event4);
    second.add_event(event5);

    bc.append(second).unwrap();
}
