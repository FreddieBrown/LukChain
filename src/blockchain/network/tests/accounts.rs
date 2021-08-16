use crate::blockchain::{
    config::Profile,
    network::{Account, Role},
    Data, Event,
};

#[test]
fn account_sign_event() {
    // Create already signed event
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
    let event: Event<Data> = user.new_event(Data::GroupMessage(String::from("Hello")));
    assert!(event.verify_sign(&user.pub_key));
}
