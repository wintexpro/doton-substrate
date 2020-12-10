#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use super::mock::{new_test_ext, SimpleMsg, Origin, Test};

#[test]
fn writeing_msg_should_work() {
  new_test_ext().execute_with(|| {
    let msg = String::from("hello");

    assert_ok!(SimpleMsg::write_msg(Origin::signed(1), 0, msg));

    let (_, _ , recived_msg) = <Messages<Test>>::get(0);

    assert_eq!(recived_msg, "hello");
  });
}
