use crate::{
    blockchain::{Data, Event},
    config::Profile,
    network::{Account, Role},
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
        },
    );
    let event: Event = user.new_event(Data::GroupMessage(String::from("Hello")));
    assert!(event.verify_sign(&user.pub_key));
}
