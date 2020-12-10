#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use super::mock::{new_test_ext, SimpleMsg, Origin, Test};

#[test]
fn writeing_msg_should_work() {
  new_test_ext().execute_with(|| {
    let msg: Vec<u8> = vec![104, 101, 108, 108, 111];

    assert_ok!(SimpleMsg::write_msg(Origin::signed(1), 0, msg));

    let (_, _ , recived_msg_vec) = <Messages<Test>>::get(0);
    let recived_msg = String::from_utf8(recived_msg_vec).unwrap();

    assert_eq!(recived_msg, "hello");
  });
}
