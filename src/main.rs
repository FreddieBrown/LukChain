use blockchat::blockchain::{
    events::{Data, Event},
    Block, BlockChain,
};
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};

fn main() {
    let mut rng = OsRng;
    let bits = 2048;
    let private_key1 = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key1 = RsaPublicKey::from(&private_key1);

    let private_key2 = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key2 = RsaPublicKey::from(&private_key2);

    println!("Declared Keys");

    let mut bc: BlockChain = BlockChain::new();

    bc.new_user(1, public_key1.clone());
    bc.new_user(2, public_key2.clone());

    println!("Added Public Keys");

    let mut genesis: Block = Block::new(None);

    genesis.set_nonce(123);

    let mut event1: Event = Event::new(1, Data::IndividualMessage(String::from("Message1")));
    event1.sign(&private_key1);
    let mut event2: Event = Event::new(1, Data::GroupMessage(String::from("Message2")));
    event2.sign(&private_key1);

    genesis.add_event(event1);
    genesis.add_event(event2);

    bc.append(genesis).unwrap();

    let mut second: Block = Block::new(bc.last_hash());

    second.set_nonce(321);

    let mut event3: Event = Event::new(2, Data::IndividualMessage(String::from("Message3")));
    event3.sign(&private_key2);
    let mut event4: Event = Event::new(2, Data::GroupMessage(String::from("Message4")));
    event4.sign(&private_key2);

    second.add_event(event3);
    second.add_event(event4);

    bc.append(second).unwrap();
}
