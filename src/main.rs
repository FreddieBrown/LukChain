use blockchat::blockchain::{
    accounts::{Account, Role},
    events::{Data, Event},
    Block, BlockChain,
};

fn main() {
    let user1 = Account::new(Role::User);
    let user2 = Account::new(Role::User);

    let mut bc: BlockChain = BlockChain::new();

    bc.new_user(user1.id, user1.pub_key.clone());
    bc.new_user(user2.id, user2.pub_key.clone());

    println!("Added Public Keys");

    let mut genesis: Block = Block::new(None);

    genesis.set_nonce(123);

    let mut event1: Event = Event::new(user1.id, Data::IndividualMessage(String::from("Message1")));
    user1.sign_event(&mut event1);
    let mut event2: Event = Event::new(user1.id, Data::GroupMessage(String::from("Message2")));
    user1.sign_event(&mut event2);

    genesis.add_event(event1);
    genesis.add_event(event2);

    bc.append(genesis).unwrap();

    let mut second: Block = Block::new(bc.last_hash());

    second.set_nonce(321);

    let mut event3: Event = Event::new(user2.id, Data::IndividualMessage(String::from("Message3")));
    user2.sign_event(&mut event3);
    let mut event4: Event = Event::new(user2.id, Data::GroupMessage(String::from("Message4")));
    user2.sign_event(&mut event4);

    second.add_event(event3);
    second.add_event(event4);

    bc.append(second).unwrap();
}
