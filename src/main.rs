use blockchat::blockchain::{
    events::{Data, Event},
    Block, BlockChain,
};

fn main() {
    let mut bc: BlockChain = BlockChain::new();

    let mut genesis: Block = Block::new(None);

    genesis.set_nonce(123);

    let event1: Event = Event::new(Data::IndividualMessage(String::from("Message1")));
    let event2: Event = Event::new(Data::GroupMessage(String::from("Message2")));

    genesis.add_event(event1);
    genesis.add_event(event2);

    bc.append(genesis).unwrap();

    let mut second: Block = Block::new(bc.last_hash());

    second.set_nonce(321);

    let event3: Event = Event::new(Data::IndividualMessage(String::from("Message3")));
    let event4: Event = Event::new(Data::GroupMessage(String::from("Message4")));

    second.add_event(event3);
    second.add_event(event4);

    bc.append(second).unwrap();
}
