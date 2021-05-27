#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use super::mock::{new_test_ext, Origin, Call, Bridge, SimpleMsg, RELAYER_A};

#[test]
fn writeing_incoming_msg_should_work() {
  new_test_ext().execute_with(|| {
    let msg: Vec<u8> = vec![104, 101, 108, 108, 111];
    let from: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let proposal = Call::SimpleMsg(crate::Call::write_msg(from, 0, msg));
    let prop_id = 1;
    let src_id = 1;
    let r_id = bridge::derive_resource_id(src_id, b"hash");
    let resource = b"SimpleMsg.write_msg".to_vec();

    assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_A));
    assert_ok!(Bridge::whitelist_chain(Origin::root(), src_id));
    assert_ok!(Bridge::set_resource(Origin::root(), r_id, resource));

    assert_ok!(Bridge::acknowledge_proposal(
      Origin::signed(RELAYER_A),
      prop_id,
      src_id,
      r_id,
      Box::new(proposal.clone())
    ));
  });
}

#[test]
fn send_msg_should_work() {
  new_test_ext().execute_with(|| {
    let msg: Vec<u8> = vec![104, 101, 108, 108, 111];
    let dest_id = 1;

    assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_id));

    assert_ok!(SimpleMsg::send_msg(
      Origin::signed(0x02),
      msg,
      dest_id,
    ));
  });
}
